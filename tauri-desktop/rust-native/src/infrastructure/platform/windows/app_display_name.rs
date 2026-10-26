use std::path::Path;

use crate::application::ports::app_display_name::AppDisplayNameResolver;

#[derive(Default)]
pub struct WindowsAppDisplayNameResolver;

impl AppDisplayNameResolver for WindowsAppDisplayNameResolver {
    fn resolve(&self, executable_name: &str) -> Option<String> {
        let name = Path::new(executable_name).file_stem()?.to_str()?.trim();
        (!name.is_empty()).then(|| name.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsAppDisplayNameResolver;
    use crate::application::ports::app_display_name::AppDisplayNameResolver;

    #[test]
    fn resolves_executable_display_name_without_extension() {
        assert_eq!(
            WindowsAppDisplayNameResolver.resolve("C:\\Apps\\Code.exe"),
            Some("Code".to_owned())
        );
    }

    #[test]
    fn rejects_empty_executable_names() {
        assert_eq!(WindowsAppDisplayNameResolver.resolve(".exe"), None);
    }
}
