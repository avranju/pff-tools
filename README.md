# pff-tools

I have some Microsoft Outlook OST files lying around that I needed to look at
from time to time. It felt like too much of a hassle to have to boot into
Windows and setup Outlook and then load the OST files into it just to search
for one mail. Turns out there's a pretty good OSS library called [libpff](https://github.com/libyal/libpff) that knows how to parse PST/OST files. I of course, want everything
in Rust, so I generated a Rust binding for `libpff`, wrote a safe wrapper library
and then a CLI tool for dealing with the files.

-   The `pff-sys` crate has the Rust bindings for `libpff`.
-   The `pff` crate is the safe and hopefully idiomatic Rust wrapper for `pff-sys`.
-   The `pff-cli` crate is the CLI tool.

## CLI Commands

The `pff-cli` tool supports the following commands.

### Index mails

You can give it a PST/OST file and have it index all the mails (optionally
including the message body) with a [Meilisearch](https://www.meilisearch.com/)
server. Here are the usage instructions.

```
pff-cli-index
Index all emails

USAGE:
    pff-cli --pff-file <PFF_FILE> index [OPTIONS] --server <SERVER> --index-name <INDEX_NAME>

OPTIONS:
    -a, --api-key <API_KEY>
            Search server API key (if any)

    -b, --include-body
            Should the message body be included in the index?

    -f, --progress-file <PROGRESS_FILE>
            File to save progress to so we can resume later [default: progress.csv]

    -h, --help
            Print help information

    -i, --index-name <INDEX_NAME>
            Index name

    -s, --server <SERVER>
            Search server URL in form "ip:port" or "hostname:port"

```

Note that including the message body in the index, depending on the size of your
PST/OST file, can result in a large index size in Meilisearch. If you have the
disk space, go for it.

### Export a mail as JSON

Once you have searched for the message you're looking for on the search server
you'll have a message ID of the form `8354_8514_32866_32930_2667556`, i.e., the
search results identify each message with a string like this. This is a sequence
of folder and message IDs that uniquely identify an item in the PST/OST file.
Once you have this, you can export the message in JSON form using the `export-message`
command. Here are the usage instructions.

```
pff-cli-export-message
Export a single message as JSON

USAGE:
    pff-cli --pff-file <PFF_FILE> export-message --id <ID>

OPTIONS:
    -h, --help       Print help information
    -i, --id <ID>    The ID of the message to export. The ID must be given as as a sequence '_'
                     delimited numbers. For example, 8354_8514_8546_7029316. This ID can be fetched
                     from the Meilisearch server search results. Note that this message ID path must
                     not include the root folder's ID which is what you get by default if you
                     indexed your emails using the `pff-cli index` command

```

Here's an example of how you might run this command.

```shell
pff-cli --pff-file /path/to/file.ost export-message --id 8354_8514_32866_32930_2667556
```

You can route the output through the [jq](https://stedolan.github.io/jq/) tool to
have the JSON nicely formatted.

```shell
pff-cli --pff-file /path/to/file.ost export-message --id 8354_8514_32866_32930_2667556 | jq

{
  "id": "2667556",
  "subject": "Subject here",
  "sender": {
    "name": "Alice",
    "email": "alice@email.com"
  },
  "recipients": [
    {
      "name": "Bob",
      "email": "bob@email.com"
    },
    {
      "name": "Pam",
      "email": "beesly@email.com"
    }
  ],
  "body": {
    "type": "html",
    "value": "... lots of HTML here ..."
  },
  "send_time": "2020-11-05T20:00:30",
  "delivery_time": "2020-11-05T20:00:39"
}
```

You can export the body into a file that you can then view in a browser like so.

```
pff-cli --pff-file /path/to/file.ost export-message --id 8354_8514_32866_32930_2667556 | jq -r '.body.value' > /tmp/mail.html
```

## Building the code

### Linux

In order to build you'll need Rust (duh!) and a working installation of `libpff`.
See the `libpff` [documentation](https://github.com/libyal/libpff/wiki/Building)
for learning how to build it. It's fairly straightforward. In my case, on my
Ubuntu box, the following worked great.

```shell
sudo apt install git autoconf automake autopoint libtool pkg-config
git clone https://github.com/libyal/libpff.git
cd libpff/
./synclibs.sh
./autogen.sh
./configure
make -j `nproc`
sudo make install
```

The binaries will by default get installed in `/usr/local`. To have the `libpff.so`
file appear in the Linux library cache you may to run the following post install.

```shell
sudo ldconfig
```

### macOS

I have been able to get this to work on macOS as well. You just have to follow
the build instructions on the `libpff` [wiki](https://github.com/libyal/libpff/wiki/Building).
