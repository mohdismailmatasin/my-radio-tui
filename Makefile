BINARY_NAME = my-radio-tui
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share/$(BINARY_NAME)

.PHONY: install uninstall release

install: release
	install -Dm755 target/release/$(BINARY_NAME) $(DESTDIR)$(BINDIR)/$(BINARY_NAME)
	install -Dm644 playlist/malaysia-radio.m3u8 $(DESTDIR)$(DATADIR)/malaysia-radio.m3u8

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BINARY_NAME)
	rm -rf $(DESTDIR)$(DATADIR)

release:
	cargo build --release
