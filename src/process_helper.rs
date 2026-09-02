use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{
    CheckTokenMembership, CreateWellKnownSid, PSID, WinBuiltinAdministratorsSid,
};
use windows::core::BOOL;

pub fn is_running_as_admin() -> bool {
    is_running_as_admin_impl().unwrap_or(false)
}

fn is_running_as_admin_impl() -> windows::core::Result<bool> {
    unsafe {
        let mut sid_buffer = [0u8; 68];
        let mut sid_size = sid_buffer.len() as u32;

        CreateWellKnownSid(
            WinBuiltinAdministratorsSid,
            None,
            Some(PSID(sid_buffer.as_mut_ptr() as *mut _)),
            &mut sid_size,
        )?;

        let sid = PSID(sid_buffer.as_mut_ptr() as *mut _);

        let mut is_member = BOOL(0);
        CheckTokenMembership(Some(HANDLE::default()), sid, &mut is_member)?;

        Ok(is_member.as_bool())
    }
}
