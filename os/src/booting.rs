/// clear bss
extern "C" {
    fn sbss();
    fn ebss();
}
pub fn clear_bss() {
    unsafe {
        (sbss as usize..ebss as usize).for_each(|a| (a as *mut u8).write_volatile(0));
    }
}


/// show LOGO
const LOGO: &str = r#"
__     __          ____      
\ \   / /         / __ \     
 \ \_/ /__   ___ | |  | |___ 
  \   / _ \ / _ \| |  | / __|
   | | (_) | (_) | |__| \__ \
   |_|\___/ \___/ \____/|___/
                            
"#;
pub fn show_logo() {
    println!("{}", LOGO);
}