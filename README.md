# Illiad-server

Simple audiobook server.

This project implements a simple fully self-hosted webserver.
It allows you to store your audiobooks on a remote server, give it metadata and then let the server organise that data to present it to a client.
With it you can download your audiobooks files from a central self hosted server, read them while keeping track of the position, and share that tracking position with multiple clients.
Please understand that this program is not indended for streaming audiobooks.
The intended method is to download the full book (or just chapter file), listen to it, then remove it from your device.
This server only goal is to make available your audiobooks to download locally on clients and to keep track of the user position.

In order to use it, you need to install this software on a server, and then to install a client on the device you want to listen on.
Note that this server implements a simple REST api to interact with the client.

This project is written in rust as a way for me to learn how to use the rust programming language.

This project is free software licensed under the GPLv3 and comes with absolutely no warranties.
That said, if you have an issue, feel free to send me an [email](paul@chambaz.xyz).

## Installation

In order to install this software, please do the following:

```bash
git clone git@git.chambaz.xyz:illiad-server.git
cd illiad-server
cargo install
```

Once that is done, you should start the program to make sure it runs without issue on your machine.

```bash
illiad-server
```

And of course, `<C-c>` to stop the program.

## Usage

Default configuration file is at `/etc/illiad-server/illiad.cfg`.

### Run this program on a server as root for systemd systems

```bash
sudo systemctl enable --now illiad.service
```

The server is now up.

### Run this program on a server as illiad user for systemd systemd

First modify the `/etc/systemd/system/illiad-server.service` file.


Then, to start the program, run:

```bash
sudo systemctl enable --now illiad-server.service
```

The server is now up.

## Organization

This program searches a directory for new files.
Please follow the following tree structure in order for your files to be discovered.

```
.
├── book1
│   ├── about.json
│   └── file.ogg
├── book2
│   ├── about.json
│   ├── 01-chapter.ogg
│   └── 02-chapter.ogg
└── book3
    ├── about.json
    ├── book1
    │   ├── 01-chapter.ogg
    │   └── 02-chapter.ogg
    ├── book2
    │   ├── 03-chapter.ogg
    │   ├── 04-chapter.ogg
    │   └── 05-chapter.ogg
    └── book3
        ├── 06-chapter.ogg
        └── 07-chapter.ogg
```

There are two ways to register a new audiobook, either rely on the searching algorithm to do work, or explicitly add the information yourself.
This is done inside the about.json file.

Here's how this would work, i will write an about.json for each of the following three books that i added in the above example.

`about.json book1`
```json
{
  "title": "book1",
  "author": "Paul Chambaz",
  "chapters": [
    {
      "number": 1,
      "file": "file.ogg"
    }
  ]
}
```

`about.json book2`
```json
{
  "title": "book2",
  "author": "Paul Chambaz",
  "chapters": [
    {
      "number": 1,
      "file": "01-chapter.ogg"
    },
    {
      "number": 2,
      "file": "02-chapter.ogg"
    }
  ]
}
```

`about.json book3`
```json
{
  "title": "book3",
  "author": "Paul Chambaz",
  "chapters": [
    {
      "number": 1,
      "file": "book1/01-chapter.ogg"
    },
    {
      "number": 2,
      "file": "book1/02-chapter.ogg"
    },
    {
      "number": 3,
      "file": "book2/03-chapter.ogg"
    },
    {
      "number": 4,
      "file": "book2/04-chapter.ogg"
    },
    {
      "number": 5,
      "file": "book2/05-chapter.ogg"
    },
    {
      "number": 6,
      "file": "book3/06-chapter.ogg"
    },
    {
      "number": 7,
      "file": "book3/07-chapter.ogg"
    },
  ]
}
```

This way you can make sure that the book is added exactly the way you want it to.
**If you do not specify a `chapters` json filed, then it will be auto-generated, but may fail by not being what you wanted.**

## Helper scripts

This program comes with a couple of helper scripts to manage audiobooks data.
In particular three useful scripts come bundled with illiad.

### make-chapters

This scripts auto generates the chapters json object from a directory, it runs the search algorithm then returns the output as a json file.
It is often a useful starting point to generate the json chapter and you have a problem with the default output.
Just run this script and tweak the result instead of having to modify the results.

To run it, just run:

```bash
illiad --make-chapters dir
```

### split-chapters 

This scripts auto generates chapter ogg files from a standard file.
Very often a user may get a mp3 file with chapters imbedded, this file with understand the starting and ending of each chapter then generate the appropriate result into chapter files.
You can also provide a list of timecode, which is useful for audiobooks available on websites like youtube where a comment adds timecodes.

```bash
illiad --split-chapters
illiad --split-chapters timecodes.txt
```

The timecode file:

```
00:00:00 Chapter 1
00:01:02 Chapter 2
```

*Please note that this scripts assumes the first line is the first chapter, the second line the second, etc.*

### add-book

This file is just a helper cli script to add a book, just run this script by pointing it a directory containing the file or a file itself.
The program will then ask you for question about the author/title, and how you want to organize your file structure, then do the job for you.
If you are unhappy about the result, please use the other scripts to change that.

```bash
illiad --add-book [dir|file]
```

## REST API

As mentionned above, this server implements a REST API.
Here will be discussed all the endpoints, so you can most likely write a simple illiad client.

### Get audiobooks

`GET /audiobooks`

Returns a list of all available books using a json format.

### Get book

`GET /audiobooks/{book_id}`

Returns book data.

### Get book chapters

`GET /audiobooks/{book_id}/chapters`

Returns book chapter list

### Get book chapter

`GET /audiobooks/{book_id}/chapters/{chapter_id}`

Returns book chapter data.

### Get book info

`GET /audiobooks/{book_id}/info`

Returns book data information.

<!-- TODO: document endpoints to create account on the server with password and get token -->

## License

This project is licensed under the GPLv3 license.
Please read the LICENSE file for more information.
