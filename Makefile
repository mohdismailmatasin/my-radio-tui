BINARY_NAME = my-radio-tui
INSTALL_PATH = /usr/local/bin

.PHONY: install uninstall release

install: release
	cp target/release/$(BINARY_NAME) $(INSTALL_PATH)/$(BINARY_NAME)
	cp -r playlist $(INSTALL_PATH)/playlist

uninstall:
	rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	rm -rf $(INSTALL_PATH)/playlist

release:
	cargo build --release