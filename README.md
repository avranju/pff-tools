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

The `pff-cli` tool supports the following commands.

## Index mails

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
