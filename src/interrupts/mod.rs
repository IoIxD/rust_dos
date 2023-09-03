/* MAIN */
/*
0x20
0x21
0x22
0x23
0x24
0x25
0x26
0x27
0x28
0x29
0x2a
0x2b
0x2c
0x2d
0x2e
0x2f
 */

/* DOS API */

use core::{
    arch::asm,
    convert::{Infallible, TryFrom},
};

/**
   On execution the call restores vectors for INTS 22h to 24h from the PSP, flushes any buffers and transfers control to the terminate handler address.

   Equivalent of CP/M BDOS call 00h. INT 21h function 4Ch is preferred.
*/
pub fn program_terminate(psp_address: u8) {
    unsafe { asm!("int 0x21", in("ah") 0x00_u8, in("dl") psp_address) }
}

/**
   Reads a character from the standard input device and echoes it to the standard output device.
   If no character is ready it waits until one is available.
   I/O can be re-directed, but prevents detection of OEF.

   Equivalent to CP/M BDOS call 01h, except that if the character is CTRL-C an INT 23h is performed.
*/
pub fn character_input(ch: u8) {
    unsafe { asm!("int 0x21", in("ah") 0x01_u8, in("dl") ch) }
}

/**
    Outputs a character to the standard output device. I/O can be re-directed, but prevents detection of 'disc full'.
*/
pub fn character_output() -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x02_u8, out("dl") ret);
    }
    ret
}

/**
   Reads a character from the current auxilliary device.

   There is no way to read the status of the serial port or to detect errors through this call, therefore most PC comms packages drive the hardware directly, hence their general incompatibility with the 512.
*/
pub fn wait_for_auxiliary_input() -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x03_u8, out("al") ret);
    }
    ret
}

/**
   Outputs a character to the current auxiliary device.

   There is no way to read the status of the serial port or to detect errors through this call. Comments as Function 3.
*/
pub fn auxiliary_output(data: u8) {
    unsafe { asm!("int 0x21", in("ah") 0x04_u8, in("dl") data) }
}

/**
   Sends a Character to the current listing device.

   If the printer is busy this call will wait until the data is sent.

   There is no way to poll the printer status in DOS.
*/
pub fn printer_output(ch: u8) {
    unsafe { asm!("int 0x21", in("ah") 0x05_u8, in("dl") ch) }
}

/**
    Reads a character from the standard input device or returns zero if no character available. Also can write a character to the current standard output device. I/O can be redirected but prevents detection of EOF on input or 'disc full' on output.

    Returns two bytes:
    byte 1: input character if console input request (DL=FF)
    bool:
        * false, if console request character available (in byte 1)
        * true, if no character is ready, and function request was console input

    This call ignores CTRL-X.
*/
pub fn direct_console_io(ch: u8) -> (u8, bool) {
    let ret1: u8;
    let ret2: u8;
    unsafe {
        asm!("
            int 0x21
            mov bl, zf
        ", in("ah") 0x06_u8, in("dl") ch, out("al") ret1, out("bl") ret2);
    }
    return (ret1, ret2 != 0);
}

/**
   Reads a character from the standard input device without echoing it to the display. If no character is ready it waits until one is available.

   This call ignores CTRL-C, use function 8 if CTRL-C processing is required. There is no CP/M equivalent.
*/
pub fn direct_console_input_without_echo() -> u8 {
    let ret: u8;
    unsafe { asm!("int 0x21", in("ah") 0x07_u8, out("al") ret) }
    ret
}

/**
   Reads a character from the standard input device without copying it to the display. If no character is ready it waits until one is available.

   If CTRL-C is detected INT 23h is executed.
*/
pub fn console_input_without_echo() -> u8 {
    let ret: u8;
    unsafe { asm!("int 0x21", in("ah") 0x08_u8, out("al") ret) }
    ret
}

/**
Writes a string to the display.

The string must be terminated by the $ character (24h), which is not transmitted. Any ASCII codes can be embedded within the string.
*/
pub fn display_string(st: &str) {
    unsafe {
        asm!("
        int 0x21
    ", in("ah") 0x09_u8, in("dx") st.as_ptr() as usize)
    }
}

/**
    Reads a string from the current input device up to and including an ASCII carriage return (0Dh), placing the received data in a user-defined buffer Input can be re directed, but this prevents detection of EOF

    The first byte of the buffer specifies the maximum number of characters it can hold (1 to 255). This value must be supplied by the user. The second byte of the buffer is set by DOS to the number of characters actually read, excluding the terminating RETURN. If the buffer fills to one less than its maximum size the bell is sounded and subsequent input is ignored.

    If a CTRL-C is detected an INT 23h is executed. Normal DOS keyboard editing is supported during input
*/
pub fn buffered_keyboard_input(st: &str) {
    unsafe {
        asm!("
        int 0x21
    ", in("ah") 0x0A_u8, in("dx") st.as_ptr() as usize)
    }
}

/**
    Checks whether a character is available from the standard input device. Input can be redirected

    Notes: if an input character is waiting this function continues to return a true flag until the character is read by a call to function 1, 6, 7, 8 or 0Ah.

*/
pub fn get_input_status() -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x0B_u8, out("al") ret);
    }
    ret
}

/**
    The allowed numbers that can be passed into flush_input_buffer_and_input
*/
pub enum InputFunction {
    /// [buffered_keyboard_input](crate::interrupts::character_input)
    CharacterInput = 0x01,
    /// [buffered_keyboard_input](crate::interrupts::direct_console_io)
    DirectConsoleIO = 0x06,
    /// [buffered_keyboard_input](crate::interrupts::direct_console_input_without_echo)
    DirectConsoleInputWithoutEcho = 0x07,
    /// [buffered_keyboard_input](crate::interrupts::console_input_without_echo)
    CharacterInputWithoutEcho = 0x08,
    /// [buffered_keyboard_input](crate::interrupts::buffered_keyboard_input)
    BufferedKeyboardInput = 0x0A,
}

/**
 * Clears the standard input buffer then invokes one of the standard input functions.
 * Returns None if buffered_keyboard_input is called, otherwise returns the character input.
 */
pub fn flush_input_buffer_and_input(inp: InputFunction) -> Option<u8> {
    let mut ret: u8;
    unsafe { asm!("int 0x21", in("ah") 0x0C_u8, inout("al") inp as u8 => ret) }
    if let InputFunction::BufferedKeyboardInput = inp {
        return None;
    } else {
        return Some(ret);
    }
}

/**
   Flush all outstanding file buffers by physically updating to disc.

   This call does *not* update disc directories for any open files.
   If the program fails to close files before the disc is removed and the files have changed size, their directory entries will be incorrect.
*/
pub fn disk_reset() {
    unsafe { asm!("int 0x21", in("ah") 0x0D_u8) }
}

pub enum DriveLetter {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Unknown,
}

impl From<u8> for DriveLetter {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            8 => Self::I,
            9 => Self::J,
            10 => Self::K,
            11 => Self::L,
            12 => Self::M,
            13 => Self::N,
            14 => Self::O,
            15 => Self::P,
            16 => Self::Q,
            17 => Self::R,
            18 => Self::S,
            19 => Self::T,
            20 => Self::U,
            21 => Self::V,
            22 => Self::W,
            23 => Self::X,
            24 => Self::Y,
            25 => Self::Z,
            _ => Self::Unknown,
        }
    }
}

/**
   Sets the specified drive to be the default drive and returns the total number of logical drives in the system.

   In the 512's DOS Plus (2.1) this call always returns five as the number of logical drives, though a maxirnum of four are supported.
*/
pub fn set_default_drive(drive_code: DriveLetter) {
    unsafe { asm!("int 0x21", in("ah") 0x0E_u8, in("dl") drive_code as u8) }
}

/**
   Opens a file and makes it available for read/write operations.
*/
// TODO: Proper FCB type.
pub fn open_file(fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x0F_u8, in("dx") fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn close_file(fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x10_u8, in("dx") fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn find_first_file(fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x11_u8, in("dx") fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn find_next_file(fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x12_u8, in("dx") fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn delete_file(fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x13_u8, in("dx") fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn sequential_read(previously_opened_fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x14_u8, in("dx") previously_opened_fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn sequential_write(previously_opened_fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x15_u8, in("dx") previously_opened_fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn create_or_truncate_file(unopened_fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x16_u8, in("dx") unopened_fcb.as_ptr() as usize) }
}

// TODO: Proper FCB type.
pub fn rename_file(special_fcb: &[u8; 36]) {
    unsafe { asm!("int 0x21", in("ah") 0x17_u8, in("dx") special_fcb.as_ptr() as usize) }
}

/*pub fn reserved() {
    unsafe { asm!("int 0x21", in("ah") 0x18_u8, in("dl") ch) }
}*/

pub fn get_default_drive() -> DriveLetter {
    let mut ret: u8;
    unsafe { asm!("int 0x21", in("ah") 0x19_u8, out("al") ret) }
    DriveLetter::from(ret)
}

// TODO: proper DTA type.
pub fn set_disk_transfer_address(dta: [u8; 43]) {
    unsafe { asm!("int 0x21", in("ah") 0x1A_u8, in("dx") dta.as_ptr() as usize) }
}

/**
   Drive allocation info gotten from either [get_allocation_info_for_default_drive](interupts::get_allocation_info_for_default_drive) or [get_allocation_info_for_specified_drive](interrupts::get_allocation_info_for_specified_drive)
*/
pub struct DriveAllocationInfo {
    /// Number of sectors
    pub sector_num: u8,
    /// Pointer to the FAT information byte. To read the contents of the FAT into memory use INT 25h. To obtain infomation about discs other than the default drive use function 1Ch. See also function 36h which returns similar data.
    pub fat_id_addr: *const u16,
    /// Sector Size
    pub sector_size: u16,
    /// Number of sectors
    pub number_of_clusters: u16,
}

/**
   Obtains selected information about the current disk drive.

   Returns None if the drive is invalid.
*/
pub fn get_allocation_info_for_default_drive() -> Option<DriveAllocationInfo> {
    let mut ret1: u8;
    let mut ret2: *const u16;
    let mut ret3: u16;
    let mut ret4: u16;
    unsafe {
        asm!("int 0x21", in("ah") 0x1B_u8, out("al") ret1, out("bx") ret2, out("cx") ret3, out("dx") ret4,)
    }
    if ret1 == 0xFF {
        None
    } else {
        Some(DriveAllocationInfo {
            sector_num: ret1,
            fat_id_addr: ret2,
            sector_size: ret3,
            number_of_clusters: ret4,
        })
    }
}

/**
   Obtains selected information about the provided drive letter.

   Returns None if the drive is invalid.
*/
pub fn get_allocation_info_for_specified_drive(
    drive_code: DriveLetter,
) -> Option<DriveAllocationInfo> {
    let mut ret1: u8;
    let mut ret2: *const u16;
    let mut ret3: u16;
    let mut ret4: u16;
    unsafe {
        asm!("int 0x21", in("ah") 0x1C_u8, in("dl") drive_code as u8, out("al") ret1, out("bx") ret2, out("cx") ret3, lateout("dx") ret4,)
    }
    if ret1 == 0xFF {
        None
    } else {
        Some(DriveAllocationInfo {
            sector_num: ret1,
            fat_id_addr: ret2,
            sector_size: ret3,
            number_of_clusters: ret4,
        })
    }
}

/*
pub fn reserved() {
    unsafe { asm!("int 0x21", in("ah") 0x1D_u8, in("dl") ch) }
}

pub fn reserved() {
    unsafe { asm!("int 0x21", in("ah") 0x1E_u8, in("dl") ch) }
}
*/

pub struct DiskParameterBlock {
    pub bytes_per_sector: u16,
    pub cluster_num: u16,
    pub media_id_byte: u16,
}

/// Get disk parameter block for default drive.
/// This was undocumented until DOS 5.0+, and I need to get my hands on the DOS programmer's manual to actually know exactly what this did.
pub fn get_disk_parameter_block_for_default_drive() -> DiskParameterBlock {
    let mut ret1: u16;
    let mut ret2: u16;
    let mut ret3: u16;
    unsafe { asm!("int 0x21", in("ah") 0x1F_u8, out("cx") ret1, out("dx") ret2, out("bx") ret3) }
    DiskParameterBlock {
        bytes_per_sector: ret1,
        cluster_num: ret2,
        media_id_byte: ret3,
    }
}

/*pub fn reserved() {
    unsafe { asm!("int 0x21", in("ah") 0x20_u8, in("dl") ch) }
}*/

/**
   Reads a selected record from an opened file.
*/
pub fn random_read(previously_opened_fcb: &[u8; 36]) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x21_u8, in("dx") previously_opened_fcb.as_ptr() as usize, out("al") ret)
    }
    ret
}

pub fn random_write(previously_opened_fcb: &[u8; 36]) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x22_u8, in("dx") previously_opened_fcb.as_ptr() as usize, out("al") ret)
    }
    ret
}

pub fn get_file_size_in_records(previously_opened_fcb: &[u8; 36]) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x23_u8, in("dx") previously_opened_fcb.as_ptr() as usize, out("al") ret)
    }
    ret
}

pub fn set_random_record_number(previously_opened_fcb: &[u8; 36]) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("int 0x21", in("ah") 0x24_u8, in("dx") previously_opened_fcb.as_ptr() as usize, out("al") ret)
    }
    ret
}

/**
    Initialises an interrupt vector to point to the supplied address.
    This is the approved way to amend interrupt vector contents.
Before changing the contents of a vector, Function 35h should be used to obtain the original entry, which should be re-instated when your code terminates. The only exceptions to this rule are interrupt vectors 22h to 24h, which are automatically restored from the PSP on program termination.
*/
pub fn set_interrupt_vector() {
    unsafe { asm!("int 0x21", in("ah") 0x25_u8, in("dl") ch) }
}

pub fn create_psp() {
    unsafe { asm!("int 0x21", in("ah") 0x26_u8, in("dl") ch) }
}

pub fn random_block_read() {
    unsafe { asm!("int 0x21", in("ah") 0x27_u8, in("dl") ch) }
}

pub fn random_block_write() {
    unsafe { asm!("int 0x21", in("ah") 0x28_u8, in("dl") ch) }
}

pub fn parse_filename() {
    unsafe { asm!("int 0x21", in("ah") 0x29_u8, in("dl") ch) }
}

pub fn get_date() {
    unsafe { asm!("int 0x21", in("ah") 0x2A_u8, in("dl") ch) }
}

pub fn set_date() {
    unsafe { asm!("int 0x21", in("ah") 0x2B_u8, in("dl") ch) }
}

pub fn get_time() {
    unsafe { asm!("int 0x21", in("ah") 0x2C_u8, in("dl") ch) }
}

pub fn set_time() {
    unsafe { asm!("int 0x21", in("ah") 0x2D_u8, in("dl") ch) }
}

pub fn set_verify_flag() {
    unsafe { asm!("int 0x21", in("ah") 0x2E_u8, in("dl") ch) }
}

pub fn get_disk_transfer_address() {
    unsafe { asm!("int 0x21", in("ah") 0x2F_u8, in("dl") ch) }
}

pub fn get_dos_version() {
    unsafe { asm!("int 0x21", in("ah") 0x30_u8, in("dl") ch) }
}

pub fn terminate_and_stay_resident() {
    unsafe { asm!("int 0x21", in("ah") 0x31_u8, in("dl") ch) }
}

pub fn get_disk_parameter_block_for_specified_drive() {
    unsafe { asm!("int 0x21", in("ah") 0x32_u8, in("dl") ch) }
}

pub fn get_or_set_ctrl_break() {
    unsafe { asm!("int 0x21", in("ah") 0x33_u8, in("dl") ch) }
}

pub fn get_in_dos_flag_pointer() {
    unsafe { asm!("int 0x21", in("ah") 0x34_u8, in("dl") ch) }
}

pub fn get_interrupt_vector() {
    unsafe { asm!("int 0x21", in("ah") 0x35_u8, in("dl") ch) }
}

pub fn get_free_disk_space() {
    unsafe { asm!("int 0x21", in("ah") 0x36_u8, in("dl") ch) }
}

pub fn get_or_set_switch_character() {
    unsafe { asm!("int 0x21", in("ah") 0x37_u8, in("dl") ch) }
}

pub fn get_or_set_country_info() {
    unsafe { asm!("int 0x21", in("ah") 0x38_u8, in("dl") ch) }
}

pub fn create_subdirectory() {
    unsafe { asm!("int 0x21", in("ah") 0x39_u8, in("dl") ch) }
}

pub fn remove_subdirectory() {
    unsafe { asm!("int 0x21", in("ah") 0x3A_u8, in("dl") ch) }
}

pub fn change_current_directory() {
    unsafe { asm!("int 0x21", in("ah") 0x3B_u8, in("dl") ch) }
}

pub fn create_or_truncate_file() {
    unsafe { asm!("int 0x21", in("ah") 0x3C_u8, in("dl") ch) }
}

pub fn open_file() {
    unsafe { asm!("int 0x21", in("ah") 0x3D_u8, in("dl") ch) }
}

pub fn close_file() {
    unsafe { asm!("int 0x21", in("ah") 0x3E_u8, in("dl") ch) }
}

pub fn read_file_or_device() {
    unsafe { asm!("int 0x21", in("ah") 0x3F_u8, in("dl") ch) }
}

pub fn write_file_or_device() {
    unsafe { asm!("int 0x21", in("ah") 0x40_u8, in("dl") ch) }
}

pub fn delete_file() {
    unsafe { asm!("int 0x21", in("ah") 0x41_u8, in("dl") ch) }
}

pub fn move_file_pointer() {
    unsafe { asm!("int 0x21", in("ah") 0x42_u8, in("dl") ch) }
}

pub fn get_or_set_file_attributes() {
    unsafe { asm!("int 0x21", in("ah") 0x43_u8, in("dl") ch) }
}

pub fn io_control_for_devices() {
    unsafe { asm!("int 0x21", in("ah") 0x44_u8, in("dl") ch) }
}

pub fn duplicate_handle() {
    unsafe { asm!("int 0x21", in("ah") 0x45_u8, in("dl") ch) }
}

pub fn redirect_handle() {
    unsafe { asm!("int 0x21", in("ah") 0x46_u8, in("dl") ch) }
}

pub fn get_current_directory() {
    unsafe { asm!("int 0x21", in("ah") 0x47_u8, in("dl") ch) }
}

pub fn allocate_memory() {
    unsafe { asm!("int 0x21", in("ah") 0x48_u8, in("dl") ch) }
}

pub fn release_memory() {
    unsafe { asm!("int 0x21", in("ah") 0x49_u8, in("dl") ch) }
}

pub fn reallocate_memory() {
    unsafe { asm!("int 0x21", in("ah") 0x4A_u8, in("dl") ch) }
}

pub fn execute_program() {
    unsafe { asm!("int 0x21", in("ah") 0x4B_u8, in("dl") ch) }
}

pub fn terminate_with_return_code() {
    unsafe { asm!("int 0x21", in("ah") 0x4C_u8, in("dl") ch) }
}

pub fn get_program_return_code() {
    unsafe { asm!("int 0x21", in("ah") 0x4D_u8, in("dl") ch) }
}

pub fn find_first_file() {
    unsafe { asm!("int 0x21", in("ah") 0x4E_u8, in("dl") ch) }
}

pub fn find_next_file() {
    unsafe { asm!("int 0x21", in("ah") 0x4F_u8, in("dl") ch) }
}

pub fn set_current_psp() {
    unsafe { asm!("int 0x21", in("ah") 0x50_u8, in("dl") ch) }
}

pub fn get_current_psp() {
    unsafe { asm!("int 0x21", in("ah") 0x51_u8, in("dl") ch) }
}

pub fn get_dos_internal_pointers_sysvars() {
    unsafe { asm!("int 0x21", in("ah") 0x52_u8, in("dl") ch) }
}

pub fn create_disk_parameter_block() {
    unsafe { asm!("int 0x21", in("ah") 0x53_u8, in("dl") ch) }
}

pub fn get_verify_flag() {
    unsafe { asm!("int 0x21", in("ah") 0x54_u8, in("dl") ch) }
}

pub fn create_program_psp() {
    unsafe { asm!("int 0x21", in("ah") 0x55_u8, in("dl") ch) }
}

pub fn rename_file() {
    unsafe { asm!("int 0x21", in("ah") 0x56_u8, in("dl") ch) }
}

pub fn get_or_set_file_date_and_time() {
    unsafe { asm!("int 0x21", in("ah") 0x57_u8, in("dl") ch) }
}

pub fn get_or_set_allocation_strategy() {
    unsafe { asm!("int 0x21", in("ah") 0x58_u8, in("dl") ch) }
}

pub fn get_extended_error_info() {
    unsafe { asm!("int 0x21", in("ah") 0x59_u8, in("dl") ch) }
}

pub fn create_unique_file() {
    unsafe { asm!("int 0x21", in("ah") 0x5A_u8, in("dl") ch) }
}

pub fn create_new_file() {
    unsafe { asm!("int 0x21", in("ah") 0x5B_u8, in("dl") ch) }
}

pub fn lock_or_unlock_file() {
    unsafe { asm!("int 0x21", in("ah") 0x5C_u8, in("dl") ch) }
}

pub fn file_sharing_functions() {
    unsafe { asm!("int 0x21", in("ah") 0x5D_u8, in("dl") ch) }
}

pub fn network_functions() {
    unsafe { asm!("int 0x21", in("ah") 0x5E_u8, in("dl") ch) }
}

pub fn network_redirection_functions() {
    unsafe { asm!("int 0x21", in("ah") 0x5F_u8, in("dl") ch) }
}

pub fn qualify_filename() {
    unsafe { asm!("int 0x21", in("ah") 0x60_u8, in("dl") ch) }
}

pub fn reserved() {
    unsafe { asm!("int 0x21", in("ah") 0x61_u8, in("dl") ch) }
}

pub fn get_current_psp() {
    unsafe { asm!("int 0x21", in("ah") 0x62_u8, in("dl") ch) }
}

pub fn get_dbcs_lead_byte_table_pointer() {
    unsafe { asm!("int 0x21", in("ah") 0x63_u8, in("dl") ch) }
}

pub fn set_wait_for_external_event_flag() {
    unsafe { asm!("int 0x21", in("ah") 0x64_u8, in("dl") ch) }
}

pub fn get_extended_country_info() {
    unsafe { asm!("int 0x21", in("ah") 0x65_u8, in("dl") ch) }
}

pub fn get_or_set_code_page() {
    unsafe { asm!("int 0x21", in("ah") 0x66_u8, in("dl") ch) }
}

pub fn set_handle_count() {
    unsafe { asm!("int 0x21", in("ah") 0x67_u8, in("dl") ch) }
}

pub fn commit_file() {
    unsafe { asm!("int 0x21", in("ah") 0x68_u8, in("dl") ch) }
}

pub fn get_or_set_media_id() {
    unsafe { asm!("int 0x21", in("ah") 0x69_u8, in("dl") ch) }
}

pub fn commit_file() {
    unsafe { asm!("int 0x21", in("ah") 0x6A_u8, in("dl") ch) }
}

pub fn reserved() {
    unsafe { asm!("int 0x21", in("ah") 0x6B_u8, in("dl") ch) }
}

pub fn extended_open_or_create_file() {
    unsafe { asm!("int 0x21", in("ah") 0x6C_u8, in("dl") ch) }
}
