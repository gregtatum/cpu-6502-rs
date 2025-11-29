use std::collections::VecDeque;

use egui::RichText;
use nes_core::{
    asm::AddressToLabel,
    cpu_6502::Cpu6502,
    opcodes::{Mode, ADDRESSING_MODE_TABLE, OPCODE_STRING_TABLE},
};

const HISTORY_LIMIT: usize = 24;
const UPCOMING_LIMIT: usize = 16;

pub enum InstructionsAction {
    StepInstruction,
    Pause,
    Resume,
}

pub struct InstructionsWindow {
    open: bool,
    executed_instructions: VecDeque<String>,
    pending_action: Option<InstructionsAction>,
}

impl InstructionsWindow {
    pub fn new() -> Self {
        Self {
            open: true,
            executed_instructions: VecDeque::new(),
            pending_action: None,
        }
    }

    pub fn take_action(&mut self) -> Option<InstructionsAction> {
        self.pending_action.take()
    }

    pub fn widget(
        &mut self,
        ctx: &egui::Context,
        cpu: &Cpu6502,
        address_to_label: Option<&AddressToLabel>,
        is_breakpoint: bool,
    ) {
        let mut open = self.open;
        egui::Window::new("Instructions")
            .open(&mut open)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let status = if is_breakpoint { "Paused" } else { "Running" };
                    ui.label(RichText::new(status).monospace());

                    if is_breakpoint {
                        if ui.button("Resume").clicked() {
                            self.pending_action = Some(InstructionsAction::Resume);
                        }
                        if ui.button("Step").clicked() {
                            self.pending_action =
                                Some(InstructionsAction::StepInstruction);
                        }
                    } else if ui.button("Pause").clicked() {
                        self.pending_action = Some(InstructionsAction::Pause);
                    }
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let entries = decode_instructions(
                        cpu,
                        address_to_label,
                        &mut self.executed_instructions,
                    );

                    for entry in entries {
                        let mut text = RichText::new(entry.text).monospace();
                        if entry.is_current {
                            text = text.strong();
                        }
                        if entry.is_history {
                            text = text.color(ui.visuals().weak_text_color());
                        }
                        ui.label(text);
                    }
                });
            });

        self.open = open;
    }
}

struct InstructionEntry {
    text: String,
    is_current: bool,
    is_history: bool,
}

fn decode_instructions(
    cpu: &Cpu6502,
    address_to_label: Option<&AddressToLabel>,
    executed_instructions: &mut VecDeque<String>,
) -> Vec<InstructionEntry> {
    let mut entries: Vec<InstructionEntry> = vec![];
    let bus = cpu.bus.borrow();
    let mut program_counter = cpu.pc;

    executed_instructions.truncate(HISTORY_LIMIT);
    for history in executed_instructions.iter().rev() {
        entries.push(InstructionEntry {
            text: history.clone(),
            is_current: false,
            is_history: true,
        });
    }

    for i in 0..UPCOMING_LIMIT {
        let instruction_pc = program_counter;
        let opcode = bus.read_u8(program_counter);
        program_counter = program_counter.wrapping_add(1);

        let opcode_display = OPCODE_STRING_TABLE[opcode as usize];
        let mode = ADDRESSING_MODE_TABLE[opcode as usize];
        let mut operand = String::new();

        let mut read_u8 = || {
            let value = bus.read_u8(program_counter);
            program_counter = program_counter.wrapping_add(1);
            value
        };

        match mode {
            Mode::Absolute
            | Mode::AbsoluteIndexedX
            | Mode::AbsoluteIndexedY
            | Mode::Indirect => {
                let low = bus.read_u8(program_counter);
                let high = bus.read_u8(program_counter.wrapping_add(1));
                program_counter = program_counter.wrapping_add(2);
                let address = u16::from_le_bytes([low, high]);

                if let Some(labels) = address_to_label {
                    if let Some(label) = labels.get(&address) {
                        operand.push_str(&format!(" {}", label));
                    }
                }

                match mode {
                    Mode::Indirect => {
                        operand.push_str(&format!(" (${address:04X})"));
                    }
                    Mode::AbsoluteIndexedX => {
                        operand.push_str(&format!(" ${address:04X},X"));
                    }
                    Mode::AbsoluteIndexedY => {
                        operand.push_str(&format!(" ${address:04X},Y"));
                    }
                    Mode::Absolute => {
                        operand.push_str(&format!(" ${address:04X}"));
                    }
                    _ => {}
                }
            }
            Mode::Immediate => operand.push_str(&format!(" #${:02X}", read_u8())),
            Mode::ZeroPage => operand.push_str(&format!(" ${:02X}", read_u8())),
            Mode::ZeroPageX => operand.push_str(&format!(" ${:02X},X", read_u8())),
            Mode::ZeroPageY => operand.push_str(&format!(" ${:02X},Y", read_u8())),
            Mode::IndirectX => operand.push_str(&format!(" (${0:02X},X)", read_u8())),
            Mode::IndirectY => operand.push_str(&format!(" (${0:02X}),Y", read_u8())),
            Mode::Relative => {
                let relative = read_u8() as i8;
                let target = (instruction_pc as i32 + relative as i32) as u16;
                if let Some(labels) = address_to_label {
                    if let Some(label) = labels.get(&target) {
                        operand.push_str(&format!(" {}", label));
                    } else {
                        operand.push_str(&format!(" {relative:+}"));
                    }
                } else {
                    operand.push_str(&format!(" {relative:+}"));
                }
            }
            Mode::Implied | Mode::None | Mode::RegisterA => {}
        }

        let text = format!("${instruction_pc:04X} {opcode_display}{operand}");
        if i == 0 {
            executed_instructions.push_front(text.clone());
        }

        entries.push(InstructionEntry {
            text,
            is_current: i == 0,
            is_history: false,
        });
    }

    entries
}
