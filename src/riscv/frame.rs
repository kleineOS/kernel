#[repr(C)]
#[derive(Debug)]
pub struct Frame {
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    fp: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
}

impl Frame {
    pub fn pretty_print(&self) {
        const RESET: &str = crate::writer::RESET;

        let column_1 = [
            ("a0", self.a0),
            ("a1", self.a1),
            ("a2", self.a2),
            ("a3", self.a3),
            ("a4", self.a4),
            ("a5", self.a5),
            ("a6", self.a6),
            ("a7", self.a7),
        ];

        let column_2 = [
            ("t0", self.t0),
            ("t1", self.t1),
            ("t2", self.t2),
            ("t3", self.t3),
            ("t4", self.t4),
            ("t5", self.t5),
            ("t6", self.t6),
        ];

        let column_4 = [
            ("s1", self.s1),
            ("s2", self.s2),
            ("s3", self.s3),
            ("s4", self.s4),
            ("s5", self.s5),
            ("s6", self.s6),
            ("s7", self.s7),
            ("s8", self.s8),
            ("s9", self.s9),
            ("s10", self.s10),
            ("s11", self.s11),
        ];

        let column_3 = [
            ("ra", self.ra),
            ("sp", self.sp),
            ("gp", self.gp),
            ("tp", self.tp),
            ("fp", self.fp),
        ];

        let max_rows = column_1
            .len()
            .max(column_2.len())
            .max(column_3.len())
            .max(column_4.len());

        for i in 0..max_rows {
            if i < column_1.len() {
                print_register(Some(column_1[i]));
            } else {
                print_register(None);
            }

            if i < column_2.len() {
                print_register(Some(column_2[i]));
            } else {
                print_register(None);
            }

            if i < column_3.len() {
                print_register(Some(column_3[i]));
            } else {
                print_register(None);
            }

            if i < column_4.len() {
                print_register(Some(column_4[i]));
            }

            crate::println!("{RESET}");
        }
    }
}

// TODO: make it so the leading 0s are grey as well
fn print_register(reg: Option<(&str, usize)>) {
    let mut value_col: &str = crate::writer::BRIGHT_MAGENTA;
    let reg_col: &str = crate::writer::LIGHT_CYAN;
    const HEX_WIDTH: usize = 16;
    if let Some((name, value)) = reg {
        if value == 0 {
            value_col = crate::writer::GREY;
        }
        crate::print!(
            "{reg_col}{name:>6} {value_col}0x{value:0>width$x}",
            width = HEX_WIDTH
        );
    } else {
        crate::print!("{:width$}", "", width = HEX_WIDTH + 9);
    }
}
