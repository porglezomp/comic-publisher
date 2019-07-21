cargo build --release --all
rmdir /Q /S target\comic-publisher
mkdir target\comic-publisher
copy target\release\*.exe target\comic-publisher
robocopy /E static target\comic-publisher\static
robocopy /E templates target\comic-publisher\templates

@echo.
@echo target\comic-publisher is ready to go. Archive and upload it!