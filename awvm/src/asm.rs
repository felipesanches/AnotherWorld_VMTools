//! Assembler for the Another World VM bytecode.
//!
//! Port of `awvm-asm.py`. Two-pass: the first pass walks every
//! instruction emitting bytes (with unresolved symbols stubbed as
//! 0x0000) and records each label's address as the current emit
//! position. The second pass re-emits with the symbol table now
//! complete.
//!
//! The output is byte-identical to the Python reference for any
//! `.asm` the Python reference accepts.  Round-trip property:
//! `awvm-disasm.py → awvm-asm.py → bytes` reproduces the original
//! bytecode exactly.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

const VIDEO2: i64 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
enum OperandValue {
    Int(i64),
    Symbol(String),
}

#[derive(Debug, Clone)]
struct Operand {
    /// Optional `key=` prefix (used for keyword operands like
    /// `id=...`, `freq=...`, etc.).
    key: Option<String>,
    /// `Var` if the operand text was wrapped in `[...]`, else `Value`.
    is_var: bool,
    value: OperandValue,
}

#[derive(Debug, Clone)]
struct Instruction {
    name: String,
    operands: Vec<Operand>,
    /// Bytes captured by the disassembler in the `;@raw=...` annotation.
    /// When present, the encoder emits these bytes verbatim instead
    /// of computing them from `name + operands`. This makes
    /// disasm → asm round-trip exact even where the canonical
    /// encoding loses information (unused opcode bits, the
    /// setPalette waste byte, etc.).
    raw: Option<Vec<u8>>,
}

/// Parse one operand token, like `[HERO_ACTION]`, `0x40`, `id=0x012c`.
fn parse_operand(token: &str) -> Operand {
    let token = token.trim();
    let (key, rest) = if let Some(eq_idx) = token.find('=') {
        (
            Some(token[..eq_idx].trim().to_owned()),
            token[eq_idx + 1..].trim(),
        )
    } else {
        (None, token)
    };

    if let (Some(start), Some(end)) = (rest.find('['), rest.find(']')) {
        let inner = rest[start + 1..end].trim();
        Operand {
            key,
            is_var: true,
            value: parse_value(inner),
        }
    } else {
        Operand {
            key,
            is_var: false,
            value: parse_value(rest),
        }
    }
}

fn parse_value(s: &str) -> OperandValue {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        if let Ok(v) = i64::from_str_radix(hex, 16) {
            return OperandValue::Int(v);
        }
    }
    if let Ok(v) = s.parse::<i64>() {
        return OperandValue::Int(v);
    }
    OperandValue::Symbol(s.to_owned())
}

/// Strip the leading mnemonic and split the rest on commas.
fn parse_common(name: &str, line: &str) -> Instruction {
    let body = match line.find(name) {
        Some(idx) => line[idx + name.len()..].trim(),
        None => "",
    };
    let operands: Vec<Operand> = if body.is_empty() {
        Vec::new()
    } else {
        body.split(',').map(parse_operand).collect()
    };
    Instruction {
        name: name.to_owned(),
        operands,
        raw: None,
    }
}

const INSTRUCTION_NAMES: &[&str] = &[
    "org",
    "bankSwitch",
    "db",
    "mov",
    "add",
    "sub",
    "jmp",
    "call",
    "ret",
    "break",
    "setPalette",
    "selectVideoPage",
    "copyVideoPage",
    "blitFramebuffer",
    "video",
    "fill",
    "je",
    "jne",
    "jge",
    "jg",
    "jle",
    "jl",
    "load",
    "setup",
    "djnz",
    "freezeChannels",
    "unfreezeChannels",
    "deleteChannels",
    "killChannel",
    "text",
    "and",
    "or",
    "shl",
    "shr",
    "play",
    "song",
    "GameOver",
];

/// Encoder state: the output buffer + the symbol table, threaded
/// through the two passes.
struct Asm {
    rom: Vec<u8>,
    address: usize,
    symbols: HashMap<String, i64>,
    second_pass: bool,
}

impl Asm {
    fn new() -> Self {
        Self {
            rom: Vec::new(),
            address: 0,
            symbols: HashMap::new(),
            second_pass: false,
        }
    }

    fn ensure_room(&mut self, addr: usize, n: usize) {
        let needed = addr + n;
        if self.rom.len() < needed {
            self.rom.resize(needed, 0);
        }
    }

    fn write_byte(&mut self, v: u8) {
        let a = self.address;
        self.ensure_room(a, 1);
        self.rom[a] = v;
        self.address += 1;
    }

    fn resolve_or_zero(&self, value: &OperandValue) -> i64 {
        match value {
            OperandValue::Int(v) => *v,
            OperandValue::Symbol(s) => {
                if let Some(v) = self.symbols.get(s) {
                    *v
                } else {
                    0
                }
            }
        }
    }

    fn byte_of(&mut self, v: &OperandValue) {
        let resolved = self.resolve_or_zero(v);
        self.write_byte((resolved & 0xFF) as u8);
    }

    fn byte_const(&mut self, v: i64) {
        self.write_byte((v & 0xFF) as u8);
    }

    fn word_of(&mut self, v: &OperandValue, negative: bool) {
        let mut resolved = self.resolve_or_zero(v);
        if negative {
            resolved = 0x10000 - resolved;
        }
        self.write_byte(((resolved >> 8) & 0xFF) as u8);
        self.write_byte((resolved & 0xFF) as u8);
    }

    fn word_const(&mut self, v: i64) {
        self.write_byte(((v >> 8) & 0xFF) as u8);
        self.write_byte((v & 0xFF) as u8);
    }
}

fn keyword_operands(instr: &Instruction) -> HashMap<String, &Operand> {
    let mut m = HashMap::new();
    for op in &instr.operands {
        if let Some(k) = &op.key {
            m.insert(k.clone(), op);
        }
    }
    m
}

fn encode(asm: &mut Asm, instr: &Instruction) {
    // Round-trip path: when the disassembler captured the original
    // bytes (`;@raw=...`), emit them verbatim. Same semantics as the
    // Python reference's encode() shortcut.
    if let Some(raw) = &instr.raw {
        for b in raw {
            asm.write_byte(*b);
        }
        return;
    }

    match instr.name.as_str() {
        "org" => { /* AW VM bytecode is always at 0x0000 */ }

        "db" => {
            for op in &instr.operands {
                asm.byte_of(&op.value);
            }
        }

        "mov" => {
            let dest = &instr.operands[0];
            let data = &instr.operands[1];
            if data.is_var {
                asm.byte_const(0x01);
                asm.byte_of(&dest.value);
                asm.byte_of(&data.value);
            } else {
                asm.byte_const(0x00);
                asm.byte_of(&dest.value);
                asm.word_of(&data.value, false);
            }
        }

        "add" => {
            let dest = &instr.operands[0];
            let data = &instr.operands[1];
            if data.is_var {
                asm.byte_const(0x02);
                asm.byte_of(&dest.value);
                asm.byte_of(&data.value);
            } else {
                asm.byte_const(0x03);
                asm.byte_of(&dest.value);
                asm.word_of(&data.value, false);
            }
        }

        "sub" => {
            let dest = &instr.operands[0];
            let src = &instr.operands[1];
            if src.is_var {
                asm.byte_const(0x13);
                asm.byte_of(&dest.value);
                asm.byte_of(&src.value);
            } else {
                asm.byte_const(0x03);
                asm.byte_of(&dest.value);
                asm.word_of(&src.value, true);
            }
        }

        "and" => {
            asm.byte_const(0x14);
            asm.byte_of(&instr.operands[0].value);
            asm.word_of(&instr.operands[1].value, false);
        }
        "or" => {
            asm.byte_const(0x15);
            asm.byte_of(&instr.operands[0].value);
            asm.word_of(&instr.operands[1].value, false);
        }
        "shl" => {
            asm.byte_const(0x16);
            asm.byte_of(&instr.operands[0].value);
            asm.word_of(&instr.operands[1].value, false);
        }
        "shr" => {
            asm.byte_const(0x17);
            asm.byte_of(&instr.operands[0].value);
            asm.word_of(&instr.operands[1].value, false);
        }

        "jmp" => {
            asm.byte_const(0x07);
            asm.word_of(&instr.operands[0].value, false);
        }
        "call" => {
            asm.byte_const(0x04);
            asm.word_of(&instr.operands[0].value, false);
        }
        "ret" => asm.byte_const(0x05),
        "killChannel" => asm.byte_const(0x11),
        "break" => asm.byte_const(0x06),
        "GameOver" => asm.byte_const(0x1B),

        "text" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x12);
            asm.word_of(&ops["id"].value, false);
            asm.byte_of(&ops["x"].value);
            asm.byte_of(&ops["y"].value);
            asm.byte_of(&ops["color"].value);
        }
        "play" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x18);
            asm.word_of(&ops["id"].value, false);
            asm.byte_of(&ops["freq"].value);
            asm.byte_of(&ops["vol"].value);
            asm.byte_of(&ops["channel"].value);
        }
        "song" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x1A);
            asm.word_of(&ops["id"].value, false);
            asm.word_of(&ops["delay"].value, false);
            asm.byte_of(&ops["pos"].value);
        }

        "freezeChannels" => {
            asm.byte_const(0x0C);
            asm.byte_of(&instr.operands[0].value);
            asm.byte_of(&instr.operands[1].value);
            asm.byte_const(0x00);
        }
        "unfreezeChannels" => {
            asm.byte_const(0x0C);
            asm.byte_of(&instr.operands[0].value);
            asm.byte_of(&instr.operands[1].value);
            asm.byte_const(0x01);
        }
        "deleteChannels" => {
            asm.byte_const(0x0C);
            asm.byte_of(&instr.operands[0].value);
            asm.byte_of(&instr.operands[1].value);
            asm.byte_const(0x02);
        }

        "djnz" => {
            asm.byte_const(0x09);
            asm.byte_of(&instr.operands[0].value);
            asm.word_of(&instr.operands[1].value, false);
        }

        name @ ("je" | "jne" | "jg" | "jge" | "jl" | "jle") => {
            let b = &instr.operands[0];
            let c = &instr.operands[1];
            let addr = &instr.operands[2];
            let mut subop: i64 = match name {
                "je" => 0,
                "jne" => 1,
                "jg" => 2,
                "jge" => 3,
                "jl" => 4,
                "jle" => 5,
                _ => unreachable!(),
            };
            let c_int = asm.resolve_or_zero(&c.value);
            if c.is_var {
                subop |= 0x80;
            } else if c_int > 0xFF {
                subop |= 0x40;
            }
            asm.byte_const(0x0A);
            asm.byte_const(subop);
            asm.byte_of(&b.value);
            if !c.is_var && c_int > 0xFF {
                asm.word_of(&c.value, false);
            } else {
                asm.byte_of(&c.value);
            }
            asm.word_of(&addr.value, false);
        }

        "setPalette" => {
            let pal = &instr.operands[0];
            asm.byte_const(0x0B);
            let pal_int = asm.resolve_or_zero(&pal.value);
            asm.word_const((pal_int << 8) | 0xFF);
        }

        "load" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x19);
            asm.word_of(&ops["id"].value, false);
        }

        "bankSwitch" => {
            let bank = &instr.operands[0];
            let bank_int = asm.resolve_or_zero(&bank.value);
            asm.byte_const(0x19);
            asm.word_const(0x3E80 | (bank_int & 0xF));
        }

        "selectVideoPage" => {
            asm.byte_const(0x0D);
            asm.byte_of(&instr.operands[0].value);
        }

        "copyVideoPage" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x0F);
            asm.byte_of(&ops["src"].value);
            asm.byte_of(&ops["dst"].value);
        }

        "blitFramebuffer" => {
            asm.byte_const(0x10);
            asm.byte_of(&instr.operands[0].value);
        }

        "fill" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x0E);
            asm.byte_of(&ops["page"].value);
            asm.byte_of(&ops["color"].value);
        }

        "setup" => {
            let ops = keyword_operands(instr);
            asm.byte_const(0x08);
            asm.byte_of(&ops["channel"].value);
            asm.word_of(&ops["address"].value, false);
        }

        "video" => {
            encode_video(asm, instr);
        }

        other => {
            eprintln!("warning: unknown instruction {other}");
        }
    }
}

fn encode_video(asm: &mut Asm, instr: &Instruction) {
    let ops = keyword_operands(instr);

    // The Python uses `offs in symbols` to detect resolution.  Here we
    // mirror that: if the offset is a symbol that we have resolved,
    // use its value; otherwise treat it as 0 in the first pass.
    let offset_op = &ops["offset"];
    let offs = match &offset_op.value {
        OperandValue::Int(v) => *v,
        OperandValue::Symbol(s) => *asm.symbols.get(s).unwrap_or(&0),
    };

    let x = &ops["x"];
    let y = &ops["y"];

    if !ops.contains_key("zoom") {
        // Compact form (opcode 0x80..).
        asm.word_const(0x8000 | ((offs >> 1) & 0x7FFF));
        asm.byte_of(&x.value);
        asm.byte_of(&y.value);
        return;
    }

    let zoom = &ops["zoom"];
    let mut opcode: i64 = 0x40;

    let video_type = asm.resolve_or_zero(&ops["type"].value);
    if video_type == VIDEO2 {
        opcode |= 0x03;
    }

    let mut operand_bytes: Vec<i64> = Vec::new();
    operand_bytes.push((offs >> 9) & 0xFF);
    operand_bytes.push((offs >> 1) & 0xFF);

    if x.is_var {
        operand_bytes.push(asm.resolve_or_zero(&x.value));
        opcode |= 0x10;
    } else {
        let xv = asm.resolve_or_zero(&x.value);
        if xv <= 0x1FF {
            opcode |= 0x20;
            operand_bytes.push(xv & 0xFF);
            if xv > 0xFF {
                opcode |= 0x10;
            }
        } else {
            operand_bytes.push((xv >> 8) & 0xFF);
            operand_bytes.push(xv & 0xFF);
        }
    }

    if y.is_var {
        operand_bytes.push(asm.resolve_or_zero(&y.value));
        opcode |= 0x04;
    } else {
        let yv = asm.resolve_or_zero(&y.value);
        if yv <= 0xFF {
            opcode |= 0x08;
            operand_bytes.push(yv);
        } else {
            operand_bytes.push((yv >> 8) & 0xFF);
            operand_bytes.push(yv & 0xFF);
        }
    }

    if zoom.is_var {
        operand_bytes.push(asm.resolve_or_zero(&zoom.value));
        opcode |= 0x01;
    } else {
        let zv = asm.resolve_or_zero(&zoom.value);
        if zv != 0x40 {
            eprintln!("ERROR! Zoom can't be a constant other than 0x40!");
        }
    }

    asm.byte_const(opcode);
    for b in operand_bytes {
        asm.byte_const(b);
    }
}

fn parse_lines(input: &str) -> (HashMap<String, i64>, Vec<(Option<String>, Instruction)>) {
    let mut symbols: HashMap<String, i64> = HashMap::new();
    let mut output: Vec<(Option<String>, Instruction)> = Vec::new();
    let mut pending_label: Option<String> = None;

    for src_line in input.lines() {
        // Extract the round-trip @raw annotation BEFORE the comment-strip
        // step swallows it. Format produced by awvm-disasm:
        // `<instr>\t;@raw=0xAA,0xBB,0xCC,...`.
        let raw_bytes = parse_raw_marker(src_line);
        let line = src_line.split(';').next().unwrap_or("").trim().to_owned();

        if line.contains("EQU") {
            if let Some(idx) = line.find("EQU") {
                let name = line[..idx].trim().to_owned();
                let value_str = line[idx + 3..].trim();
                if let OperandValue::Int(v) = parse_value(value_str) {
                    symbols.insert(name, v);
                }
            }
            continue;
        }

        // Label: a token ending in `:` at the start of the (whitespace-stripped) line.
        let mut effective_line = line.clone();
        let first_token = effective_line.split_whitespace().next().unwrap_or("");
        if first_token.contains(':') {
            // Python uses `":" in line.split(" ")[0]` — i.e. checks the
            // first space-separated token. Mirror that.
            let space_split: &str = effective_line.split(' ').next().unwrap_or("");
            if space_split.contains(':') {
                let label = space_split.split(':').next().unwrap_or("").to_owned();
                pending_label = Some(label);
                if let Some(after) = effective_line.split_once(':') {
                    effective_line = after.1.to_owned();
                }
            }
        }

        for name in INSTRUCTION_NAMES {
            if effective_line.trim().starts_with(name) {
                let mut instr = parse_common(name, &effective_line);
                instr.raw = raw_bytes.clone();
                output.push((pending_label.take(), instr));
                break;
            }
        }
    }

    (symbols, output)
}

/// Parse a `;@raw=0xAA,0xBB,...` marker out of one line. Returns
/// `None` if the marker is absent.
fn parse_raw_marker(line: &str) -> Option<Vec<u8>> {
    const MARKER: &str = ";@raw=";
    let idx = line.find(MARKER)?;
    let rest = &line[idx + MARKER.len()..];
    let rest = match rest.find(';') {
        Some(end) => &rest[..end],
        None => rest,
    };
    let mut out = Vec::new();
    for tok in rest.split(',') {
        let tok = tok.trim();
        if tok.is_empty() {
            continue;
        }
        let parsed = if let Some(hex) = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")) {
            u8::from_str_radix(hex, 16).ok()?
        } else {
            tok.parse::<u8>().ok()?
        };
        out.push(parsed);
    }
    Some(out)
}

/// Assemble `input.asm` to `output.bin`. Output bytes are written
/// to `output_path`; the contents are byte-identical to what the
/// Python `awvm-asm.py` produces for the same input.
pub fn assemble(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let src = fs::read_to_string(input_path)?;
    let (initial_symbols, instructions) = parse_lines(&src);

    let mut asm = Asm::new();
    asm.symbols = initial_symbols;

    // First pass — labels resolve to current address as we go.
    asm.address = 0;
    for (label, instruction) in &instructions {
        if let Some(l) = label {
            asm.symbols.insert(l.clone(), asm.address as i64);
        }
        encode(&mut asm, instruction);
    }

    // Second pass — symbols are now complete; re-emit.
    asm.second_pass = true;
    asm.rom.clear();
    asm.address = 0;
    for (label, instruction) in &instructions {
        if let Some(l) = label {
            asm.symbols.insert(l.clone(), asm.address as i64);
        }
        encode(&mut asm, instruction);
    }

    fs::write(output_path, &asm.rom)?;
    Ok(())
}
