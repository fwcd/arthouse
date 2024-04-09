# Arthouse

[![crates.io](https://img.shields.io/crates/v/arthouse)](https://crates.io/crates/arthouse)
[![Build](https://github.com/fwcd/arthouse/actions/workflows/build.yml/badge.svg)](https://github.com/fwcd/arthouse/actions/workflows/build.yml)

Art-Net (DMX over UDP/IP) adapter for Project Lighthouse.

Using this adapter, the Project Lighthouse display can be controlled like a standard DMX fixture from lighting controllers such as [QLC+](https://www.qlcplus.org/).

## Getting Started

Make sure that `LIGHTHOUSE_USER` and `LIGHTHOUSE_TOKEN` contain valid Project Lighthouse credentials, then run

```sh
cargo run
```

Alternatively you can also pass the corresponding command-line options, see `--help` for a detailed overview. By default, the adapter will listen on `0.0.0.0:6454`, i.e. the default Art-Net port, for UDP packets.

The following guide explains how to configure QLC+, though any other Art-Net client can be used too. First, make sure that you have at least three universes that output to Art-Net on localhost. This can be configured in the Input/Output tab:

![Input/Output](screenshots/getting-started/01-input-output.png)

Then switch to the Fixtures tab and add an RGB panel fixture:

![Fixtures](screenshots/getting-started/02-fixtures.png)

The fixture needs to have 28 columns, 14 rows and "Zig Zag" displacement:

![Add RGB Panel](screenshots/getting-started/03-add-rgb-panel.png)

After confirming, RGB panels and a corresponding fixture group should have been added:

![Fixtures](screenshots/getting-started/04-fixtures.png)

Finally, switch to the Functions tab and create an RGB matrix by clicking the corresponding button in the toolbar:

![Functions](screenshots/getting-started/05-functions.png)

Select the previously created fixture group in the drop down menu on the right and click the play button to preview the animation:

![Functions](screenshots/getting-started/06-functions.png)

If everything went well, the output should be mirrored to the lighthouse:

![Lighthouse](screenshots/getting-started/07-lighthouse.png)
