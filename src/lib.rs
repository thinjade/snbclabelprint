use encoding::all::GB18030;
use encoding::{EncoderTrap, Encoding};
use libc::{c_char, c_int, c_void};
use libloading::{Library, Symbol};
use std::alloc::{alloc, Layout};

pub enum PrintLanguage {
    DEFAULT,
    BPLA,
    BPLC,
    BPLZ,
}
pub enum PortType {
    COM,
    USB,
    NET,
    WIFI,
    BLE,
    BLUETOOTH,
    FILE,
}
pub enum BarcodeType {
    CODE128,
    CODE39,
    CODE93,
    EAN8,
    EAN13,
    CODEBAR,
    ITF25,
    UPCA,
    UPCE,
}
pub enum PaperMode {
    Gaps,
    Continuous,
    Mark,
}
pub enum PrintMethod {
    Thermal,
    Transfer,
}

pub fn discover_printer(port_type: PortType) -> String {
    let port_type_int = match port_type {
        PortType::COM => 1,  
        PortType::USB => 3,  
        PortType::NET => 4,  
        PortType::WIFI => 5, 
        PortType::BLE => 6,
        PortType::BLUETOOTH => 7,
        PortType::FILE => 8, 
    };
    let slib = unsafe { Library::new("LabelPrinterSDK.dll").unwrap() };
    let dev_info = unsafe { alloc(Layout::from_size_align(8, 128).unwrap()) as *mut c_char };
    let dev_count = unsafe { alloc(Layout::new::<c_int>()) as *mut c_int };
    let buff_len: c_int = 16;
    //let dev_count_p = dev_count.as_ptr() as *mut c_int;
    let discover_printer: Symbol<
        unsafe extern "C" fn(c_int, *mut c_char, *const c_int, *mut c_int, c_int) -> c_int,
    > = unsafe { slib.get(b"DiscoverPrinter\0").unwrap() };
    let mut devs = String::new();
    unsafe {
        discover_printer(port_type_int, dev_info, &buff_len, dev_count, 0);
        for i in 0..buff_len as isize {
            if *dev_info.offset(i) as u8 as char != '\0' {
                devs.push(*dev_info.offset(i) as u8 as char);
            } else {
                break;
            };
        }
    };
    let mut dev_list: Vec<&str> = devs.split("@").collect();
    dev_list.pop();
    println!("{:?}", dev_list);
    println!("{}", unsafe { *dev_count.offset(0) as c_int });
    devs
}

#[derive(Debug)]
pub struct Printer {
    handle: *mut *mut c_void,
    slib: Library,
}
impl Printer {
    pub fn new(
        port_type: PortType,
        dev_port_info: &str,
        printer_language: PrintLanguage,
    ) -> Result<Printer, Box<dyn std::error::Error>> {
        let slib = unsafe { Library::new("LabelPrinterSDK.dll").unwrap() };
        let connect_printer: Symbol<
            unsafe extern "C" fn(c_int, *const char, c_int, *mut *mut c_void) -> c_int,
        > = unsafe { slib.get(b"ConnectPrinter\0").unwrap() };
        let ndev_port_info = format!("{}\0", dev_port_info);
        //void**
        let handle: *mut *mut c_void = unsafe { alloc(Layout::new::<c_int>()) as *mut *mut c_void };
        let pdev_port_info: *const char = ndev_port_info.as_ptr() as *const char;
        let pri_lan = match printer_language {
            PrintLanguage::DEFAULT => 0,
            PrintLanguage::BPLZ => 2, //zpl
            PrintLanguage::BPLC => 4, //cpcl
            PrintLanguage::BPLA => 6,
        };
        let port_type_int = match port_type {
            PortType::COM => 1,  
            PortType::USB => 3,  
            PortType::NET => 4,  
            PortType::WIFI => 5, 
            PortType::BLE => 6,
            PortType::BLUETOOTH => 7,
            PortType::FILE => 8, 
        };
        unsafe {
            let i = connect_printer(port_type_int, pdev_port_info, pri_lan, handle);
            println!("connect status:{:?}", i);
        }

        Ok(Printer {
            handle: handle,
            slib,
        })
    }

    pub fn set_labelsize(&self, width: c_int, height: c_int) -> c_int {
        let set_labelsize: Symbol<unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int> =
            unsafe { self.slib.get(b"SetLabelSize\0").unwrap() };
        unsafe { set_labelsize(*(self.handle), width, height) }
    }
    pub fn set_print_direction(&self, direction: c_int) -> c_int {
        let set_print_direction: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.slib.get(b"SetPrintDirection\0").unwrap() };
        unsafe { set_print_direction(*(self.handle), direction) }
    }
    pub fn print_text(
        &self,
        x: c_int,
        y: c_int,
        font_name: &str,
        text: &str,
        angle: c_int,
        font_size_h: c_int,
        font_size_v: c_int,
    ) -> c_int {
        let print_text: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                c_int,
                c_int,
                *const char,
                *const char,
                c_int,
                c_int,
                c_int,
                c_int,
            ) -> c_int,
        > = unsafe { self.slib.get(b"PrintText\0").unwrap() };
        let nfont_name = format!("{}\0", font_name);
        let pfont_name: *const char = nfont_name.as_ptr() as *const char;
        let ntext = format!("{}\0", text);
        let nntext = GB18030.encode(&ntext, EncoderTrap::Strict).unwrap();
        let ptext: *const char = nntext.as_ptr() as *const char;
        unsafe {
            print_text(
                *(self.handle),
                x,
                y,
                pfont_name,
                ptext,
                angle,
                font_size_h,
                font_size_v,
                1,
            )
        }
    }
    pub fn print_truetype_text(
        &self,
        x: c_int,
        y: c_int,
        font_name: &str,
        font_width: c_int,
        font_height: c_int,
        text: &str,
        angle: c_int,
        style: c_int,
    ) -> c_int {
        let print_truetype_text: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                c_int,
                c_int,
                *const char,
                c_int,
                c_int,
                *const char,
                c_int,
                c_int,
            ) -> c_int,
        > = unsafe { self.slib.get(b"PrintTrueTypeText\0").unwrap() };
        let nfont_name = format!("{}\0", font_name);
        let pfont_name: *const char = nfont_name.as_ptr() as *const char;
        let ntext = format!("{}\0", text);
        let nntext = GB18030.encode(&ntext, EncoderTrap::Strict).unwrap();
        let ptext: *const char = nntext.as_ptr() as *const char;
        unsafe {
            print_truetype_text(
                *(self.handle),
                x,
                y,
                pfont_name,
                font_width,
                font_height,
                ptext,
                angle,
                style,
            )
        }
    }
    pub fn print_barcode_qr(
        &self,
        x: c_int,
        y: c_int,
        content: &str,
        ecc_lever: char,
        cell_width: c_int,
        model: c_int,
    ) -> c_int {
        let print_barcode_qr: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                c_int,
                c_int,
                c_int,
                *const char,
                c_int,
                c_char,
                c_int,
                c_int,
            ) -> c_int,
        > = unsafe { self.slib.get(b"PrintBarcodeQR\0").unwrap() };
        let n_content = format!("{}\0", content);
        let p_content: *const char = n_content.as_ptr() as *const char;
        let content_len = content.len();
        unsafe {
            print_barcode_qr(
                *(self.handle),
                x,
                y,
                0,
                p_content,
                content_len as c_int,
                ecc_lever as c_char,
                cell_width,
                model,
            )
        }
    }
    pub fn print_barcode(
        &self,
        x: c_int,
        y: c_int,
        barcode_type: BarcodeType,
        rotate: c_int,
        content: &str,
        height: c_int,
        HRI: c_int,
        narrow_bar_width: c_int,
        wide_bar_width: c_int,
    ) -> c_int {
        let print_barcode: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                c_int,
                c_int,
                c_int,
                c_int,
                *const char,
                c_int,
                c_int,
                c_int,
                c_int,
            ) -> c_int,
        > = unsafe { self.slib.get(b"PrintBarcode1D\0").unwrap() };
        let n_content = format!("{}\0", content);
        let p_content: *const char = n_content.as_ptr() as *const char;
        let barcode_type_int = match barcode_type {
            BarcodeType::CODE128 => 1,
            BarcodeType::CODE39 => 2,
            BarcodeType::CODE93 => 3,
            BarcodeType::EAN8 => 4,
            BarcodeType::EAN13 => 5,
            BarcodeType::CODEBAR => 6,
            BarcodeType::ITF25 => 7,
            BarcodeType::UPCA => 8,
            BarcodeType::UPCE => 9,
        };
        unsafe {
            print_barcode(
                *(self.handle),
                x,
                y,
                barcode_type_int,
                rotate,
                p_content,
                height,
                HRI,
                narrow_bar_width,
                wide_bar_width,
            )
        }
    }
    pub fn print_rectangle(
        &self,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        thickness: c_int,
    ) -> c_int {
        let print_rectangle: Symbol<
            unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int, c_int) -> c_int,
        > = unsafe { self.slib.get(b"PrintRectangle\0").unwrap() };
        unsafe { print_rectangle(*(self.handle), x, y, width, height, thickness) }
    }

    pub fn print_line(
        &self,
        start_x: c_int,
        start_y: c_int,
        end_x: c_int,
        end_y: c_int,
        thickness: c_int,
    ) -> c_int {
        let print_line: Symbol<
            unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int, c_int) -> c_int,
        > = unsafe { self.slib.get(b"PrintLine\0").unwrap() };
        unsafe { print_line(*(self.handle), start_x, start_y, end_x, end_y, thickness) }
    }

    pub fn print_imagefile(&self, x: c_int, y: c_int, image_path: &str) -> c_int {
        let print_imagefile: Symbol<
            unsafe extern "C" fn(*mut c_void, c_int, c_int, *const char) -> c_int,
        > = unsafe { self.slib.get(b"PrintImageFile\0").unwrap() };
        let p_image_path = format!("{}\0", image_path).as_ptr() as *const char;
        unsafe { print_imagefile(*(self.handle), x, y, p_image_path) }
    }
    /***
    fn download_image() {}
    ***/
    pub fn print_label(&self, label_num: c_int) -> c_int {
        let print_label: Symbol<unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int> =
            unsafe { self.slib.get(b"PrintLabel\0").unwrap() };
        unsafe { print_label(*(self.handle), label_num, 1) }
    }
    pub fn feed_label(&self) -> c_int {
        let feed_label: Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
            unsafe { self.slib.get(b"FeedLabel\0").unwrap() };
        unsafe { feed_label(*(self.handle)) }
    }
    pub fn set_print_density(&self, density: c_int) -> c_int {
        let set_print_density: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.slib.get(b"SetPrintDensity\0").unwrap() };
        unsafe { set_print_density(*(self.handle), density) }
    }
    pub fn set_paper_mode(&self, mode: PaperMode) -> c_int {
        let p_mode = match mode {
            PaperMode::Gaps => 1,
            PaperMode::Continuous => 2,
            PaperMode::Mark => 3,
        };
        let set_paper_mode: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.slib.get(b"SetPaperMode\0").unwrap() };
        unsafe { set_paper_mode(*(self.handle), p_mode) }
    }
    pub fn set_print_method(&self, method: PrintMethod) -> c_int {
        let p_method = match method {
            PrintMethod::Thermal => 1,
            PrintMethod::Transfer => 2,
        };
        let set_print_method: Symbol<unsafe extern "C" fn(*mut c_void, c_int) -> c_int> =
            unsafe { self.slib.get(b"SetPrintMethod\0").unwrap() };
        unsafe { set_print_method(*(self.handle), p_method) }
    }
    pub fn disconnect_printer(&self) -> c_int {
        let disconnect_printer: Symbol<unsafe extern "C" fn(*mut c_void) -> c_int> =
            unsafe { self.slib.get(b"DisconnectPrinter\0").unwrap() };

        unsafe { disconnect_printer(*(self.handle)) }
    }
    pub fn get_printer_info(&self, info_id: c_int) -> String {
        let buffer = unsafe { alloc(Layout::from_size_align(8, 128).unwrap()) as *mut c_char };
        let buff_len = 64;
        let get_printer_info: Symbol<
            unsafe extern "C" fn(*mut c_void, c_int, *mut c_char, c_int) -> c_int,
        > = unsafe { self.slib.get(b"GetPrinterInfo\0").unwrap() };
        let mut buff_content = String::new();
        unsafe {
            get_printer_info(*(self.handle), info_id, buffer, buff_len);
            for i in 0..buff_len as isize {
                if *buffer.offset(i) as u8 as char != '\0' {
                    buff_content.push(*buffer.offset(i) as u8 as char);
                } else {
                    break;
                };
            }
        }
        //println!("{:#?}", buff_content);
        buff_content
    }
}
