illiad:
	@rustup default stable
	@cargo build --release

install: illiad
	@mkdir -p /usr/share/illiad
	@chmod 777 /usr/share/illiad
	@cp _database.sqlite /usr/share/illiad/database.sqlite
	@chmod 666 /usr/share/illiad/database.sqlite
	@strip target/release/illiad
	@cp target/release/illiad /usr/bin/illiad
	@cp illiad.service /usr/lib/systemd/system/illiad.service
	@systemctl daemon-reload

