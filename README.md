# Illiad

Simple audiobook server.

This project implements a simple self-hosted audiobook REST API.
It allows you to store your audiobook in once place, give it metadata there and then let the server organise that data to present it to a client.
With it you can download your audiobook files from your server, listen to them while keeping track of the position and share that tracking data on multiple clients.
Please understand that this program is not indended for streaming audiobooks. It just downloads the file on your client, the client does all the managing of the files, including deleting the file from the client once you are done with it.
This server's objective is just to centralize your collection and your position in the audiobook to you can sync them between different clients.

In order to use it, you need to install this software on a server, and then to install a client on the device you want to listen on.

This program is written in rust as a way for me to learn how to use the rust programming language.

This project is free software licensed under the GPLv3 and comes with absolutely no warranties.
That said, if you have an issue, feel free to send me an [email](paul@chambaz.xyz).

If you're interested in this project, please check out: [odyssey](https://git.chambaz.xyz/odyssey) and [odyssey-droid](https://git.chambaz.xyz/odyssey-droid), clients i wrote for this server for linux tuis and android.

## Installation

In order to install this software, please do the following:

```bash
git clone https://git.chambaz.xyz/illiad.git
cd illiad
sudo rustup default stable
sudo cargo make install
```

You will need to install `cargo-make` to use `cargo make`, to do so, run:

```
cargo install --force cargo-make
```

Once that is done, you should start the program to make sure it runs without issue on your machine.

To uninstall the program, you can also do:

```
sudo cargo make uninstall
```

**This will delete all configuration files and database, so be careful with it.**

## Usage

You should try to see if everything was installed correctly:

```bash
illiad
```

Normally the program should not work and give you a guide on how to configure it. Here, I will give the same guide.

Default configuration file is at `/etc/illiad/illiadrc`.

You can copy this configuration to `~/.config/illiad/illiadrc`. As the program will attempt to search for the file here before reverting to the default configuration. Finally if you provice `-c config.toml`, the program will read that config. Here is a list of all default parameters.

```toml
data = ""
sql = "/usr/share/illiad/database.sqlite"
port = 15000
address = "127.0.0.1"
register = true
```

All fields will take their default value in the program. If you delete `/etc/illiad/illadrc` and you do not provide any other configuration, then the program will not work.

### data

The path to the data, this is where you store all your audiobooks.

### database

The path to the sqlite3 database.

### port

The port used to run this server.

### address

The address at which the server should be run, you most likely want to keep this at its default value.

### register

Whether or not the register endpoint is allowed.

If you have changed the location of the database in your config file, make sure to create a database with sqlite3:

```
sqlite3 database.sqlite '.database'
```

**Make sure that the path you provide in your config match the location of the database. Make also sure that the directory in which the database is has write access and that the database itself has write access to the account running illiad.**

Finally, if you have done everything there is a systemd service installed, so you can just:

```
sudo systemctl start illiad.service
```

Then you can hide this behind an nginx reverse proxy (apache also works):

```
server {
    server_name illiad.your-domain-name.com;
    listen 80;
    listen [::]:80;
    location / {
        proxy_pass http://127.0.0.1:15000;
    }
}
```

```bash
nginx -t # reloads nginx
certbot --nginx # gives https certificate
ufw allow 15000 # if you have a firewall
```

Make sure to route your port 443 to your server on your router's firewall.

This will allow you to query 'https://illiad.your-domain-name.com/audiobooks' from anywhere.

## Organization

This program does not manage anything in regards to the book organisation. That said there are a couple that you need to know to be able for the program to detect your books. First, at the root of every directory that there is an audiobook in, you should include a info.toml. Here's an example:

```toml
title = "The Ancient City"
author = "Fustel de Coulanges"
```

Please note that I have created a simple format that you may want to follow for the rest of the document:

```toml
title = "The Ancient City"
author = "Fustel de Coulanges"

[0]
file = "01.ogg"
name = "Introduction -  Necessity of Studying the Oldest Beliefs of the Ancients"

[1]
file = "02.ogg"
name = "Book 1 Chapter 1 - Notions About the Soul & Death"

[2]
file = "03.ogg"
name = "Book 1 Chapter 2 - The Worship of the Dead"
```

Although this structure is completely up to you, this is the pattern that I use for odyssey, a client I wrote for this program. If you use another client, you should check how they want to organize your files, but this is what I recommend.

Please note that you do not need to do anything for your new books to be detected, the program runs in the background once every minute and adds new books when you add them.

## Endpoints

Here is a list of endpoints:

**For authentification you need to provide a header `Auth: your-api-key`.**

### GET /audiobooks

**Requires authentification.**
This endpoint gives a list of all audiobooks installed on the server. This is used for browsing your collection before downloading an audiobook.

### GET /audiobook/{hash}

**Requires authentification.**
This endpoint downloads an audiobook to the device as a .tar.gz. The client will need to uncompress the file so it can be used.

### GET /audiobook/{hash}/position

**Requires authentification.**
This endpoint gets what file and the exact position in the file that you are at in a specific book. Note that this endpoint is user variant.

### POST /audiobook/{hash}/position

**Requires authentification.**
This endpoint posts what file and exact position in the file that you are at in a specific book. Note that this endpoint is user variant.

### POST /register

This endpoint allows you to register a new user that then gets a api key using a username and password. You will need this api key for all other connections with the database.

You can remove this audiobook if you prefer to add these manually, this is done in the configuration file. This will limit who can join your server.

```
register = false
```

You will need to modify your sqlite3 database yourself to add new users. This is done with.

```bash
sqlite3 database.sqlite "INSERT INTO accounts (user, password, key) VALUES ($user, $password, $key);"
```

Where user is their username, password is their encrypted password (check with client for encryption method - odyssey uses mkpasswd with md5) and key is their api key. You can generate a key with ssl.

```bash
mkpasswd -m md5 'your-password' # for the password
openssl rand -base64 16 # for api key
```

### POST /login

This endpoint allows you to recuperate a user's api key using a username and password. You will need this api key for all other connections with the database.

## License

This project is licensed under the GPLv3 license.
Please read the LICENSE file for more information.
