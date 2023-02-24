illiad:
	@cargo build --release

install: illiad
	@mkdir -p /etc/illiad
	@chmod 755 /etc/illiad
	@cp illiadrc /etc/illiad
	@chmod 644 /etc/illiad/illiadrc
	@mkdir -p /usr/share/illiad
	@chmod 755 /usr/share/illiad
	@sqlite3 /usr/share/illiad/database.sqlite '.database'
	@chmod 644 /usr/share/illiad/database.sqlite
	@strip target/release/illiad
	@cp target/release/illiad /usr/bin/illiad
	@cp illiad.service /usr/lib/systemd/system/illiad.service
	@systemctl daemon-reload

