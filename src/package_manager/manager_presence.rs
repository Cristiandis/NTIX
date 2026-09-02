pub trait ManagerPresence {
    fn is_chocolatey_installed(&self) -> bool;
    fn is_scoop_installed(&self) -> bool;
}