use {
    crate::tss,
    conquer_once::spin::OnceCell,
    x86_64::{
        instructions::{
            segmentation::{Segment, CS, DS, ES, FS, GS, SS},
            tables::load_tss,
        },
        structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    },
};

static GDT: OnceCell<GlobalDescriptorTable> = OnceCell::uninit();

static SELECTORS: OnceCell<Selectors> = OnceCell::uninit();

struct Selectors {
    kernel_code: SegmentSelector,
    kernel_data: SegmentSelector,
    user_code: SegmentSelector,
    user_data: SegmentSelector,
    tss: SegmentSelector,
}

pub(super) fn kernel_code_selector() -> SegmentSelector {
    selectors().kernel_code
}

pub(super) fn kernel_data_selector() -> SegmentSelector {
    selectors().kernel_data
}

pub(super) fn user_code_selector() -> SegmentSelector {
    selectors().user_code
}

pub(super) fn user_data_selector() -> SegmentSelector {
    selectors().user_data
}

pub(super) fn init() {
    init_gdt();
    load();
    load_segments();

    #[cfg(test_on_qemu)]
    tests::main();
}

fn init_gdt() {
    let r = GDT.try_init_once(|| {
        let mut gdt = GlobalDescriptorTable::new();

        let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_data = gdt.add_entry(Descriptor::user_data_segment());
        let user_code = gdt.add_entry(Descriptor::user_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(unsafe { tss::as_ref() }));

        init_selectors(Selectors {
            kernel_code,
            kernel_data,
            user_code,
            user_data,
            tss,
        });

        gdt
    });
    r.expect("Failed to initialize GDT.");
}

fn init_selectors(selectors: Selectors) {
    let r = SELECTORS.try_init_once(|| selectors);
    r.expect("Failed to initialize `SELECTORS`.");
}

fn load() {
    gdt().load();

    unsafe {
        load_tss(selectors().tss);
    }
}

fn load_segments() {
    let selectors = selectors();

    let code = selectors.kernel_code;
    let data = selectors.kernel_data;

    unsafe {
        CS::set_reg(code);

        DS::set_reg(data);
        ES::set_reg(data);
        FS::set_reg(data);
        GS::set_reg(data);
        SS::set_reg(data);
    }
}

fn gdt<'a>() -> &'a GlobalDescriptorTable {
    let gdt = GDT.try_get();
    gdt.expect("GDT is not initialized.")
}

fn selectors<'a>() -> &'a Selectors {
    let selectors = SELECTORS.try_get();
    selectors.expect("`SELECTORS` is not initialized.")
}

#[cfg(test_on_qemu)]
mod tests {
    use {
        super::{gdt, selectors},
        x86_64::{
            instructions::{
                segmentation::{Segment, CS, DS, ES, FS, GS, SS},
                tables,
            },
            VirtAddr,
        },
    };

    pub(super) fn main() {
        assert_gdt_address_is_correct();
        assert_selectors_are_correctly_set();
    }

    fn assert_gdt_address_is_correct() {
        let gdt = gdt();
        let expected_addr = VirtAddr::from_ptr(gdt);

        let descriptor_table_ptr = tables::sgdt();
        let actual_addr = descriptor_table_ptr.base;

        assert_eq!(
            expected_addr, actual_addr,
            "The address of the current GDT is not correct."
        );
    }

    fn assert_selectors_are_correctly_set() {
        let selectors = selectors();

        let code = selectors.kernel_code;
        let data = selectors.kernel_data;

        macro_rules! assert_segment {
            ($seg:ident,$correct:expr) => {
                assert_eq!(
                    $seg::get_reg(),
                    $correct,
                    "Incorrect {}",
                    core::stringify!($seg)
                );
            };
        }

        macro_rules! code{
            ($($seg:ident),+)=>{
                $(assert_segment!($seg,code);)+
            }
        }

        macro_rules! data{
            ($($seg:ident),+)=>{
                $(assert_segment!($seg,data);)+
            }
        }

        code!(CS);

        data!(DS, ES, FS, GS, SS);
    }
}
