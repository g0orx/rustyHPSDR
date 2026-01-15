# rustyHPSDR

My first attempt at a Rust application.

This is very early code to implement a UI in Rust using gtk4-rs to implement a Radio for the OpenHPSDR Protocols.

The current code only implements a Receiver. It does work with Protocol 1 and Protocol 2 radios including the Hermes Lite. I will be adding more features including Transmit over the next few weeks.

Transmit is still in progress. It is working for protocol 2 but needs more work for protocol 1.

WDSP is now included in the source tree and is built as a static library, so there is no need to dowenload and install WDSP any more.

You will need to install Rust and Cargo. See [Rust install](https://www.rust-lang.org/tools/install) for information on installing Rust and Cargo.

This version of the code has many major changes. A major change is that the UI is now 3 xml files: ui.xml, discovery.xml. configure.xml that are used at run time to build the ui interface.

Please note that not all images or the current UI. My antenna came down in a recent storm. This is the latest UI ...

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/latest.png">

# When the application is run it will first discovery all the HPSDR compatable devices on the network interfaces.

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/discovery.png">

# When a device is selected and Start button clicked the radio will start running

Currently Split, Mic Gain and Drive do not do anything. They will once transmit is implemented.

Most of the other buttons are self explanitary.

By default when first started both RX1 and RX2 receivers are displayed. The currently active receiver is indicated by the spectrum display having the lighter blue background. When enabled, clicking on the spectru or waterfall will more the receiver to tht frequency.  If you click on spectrum or waterfall display of the receiver with the darker blue spectrum background it will become the active receiver and further clicks will change the frequency.

Note that the scroll wheel will change frequency of the receiver that the mouse cursor is over.

The scroll wheel can be used to change frequency when the curosr is in the VFO window, the Spectrum window and Waterfall window.

The CTUN button enables Click Tuning. When active it allows you to click within the Spectrum or Waterfall to QSY without shifting the Spectrum or Waterfall display. When "CTUN" is inactive, clicking on the Spectrum re-centers the Spectrum, thereby shifting the entire Spectrum. 

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/screenshot1.png">

# Adjusting spectrum and waterfall display

If you move the mouse over to the left side of the spectrum display the cursor will change to an up arrow, down arrow or up/down arrow that will allow you to use the scroll wheel to move the limits up and down for the current band. When the up arrow shows it moves the upper limiit, when the down arrow shows it moves the lower limit and when the up/down arrow shows it moves the upper and lower limits.

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/cursor.png">

# Zoom and Pan

I have now added the first attempt at Zoom and Pan

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/zoom1.png">


# Please look at the Wiki (tab at top of page) for instructions describing how to build the application and also how to configure the system (tested with Ubuntu 25.10) to run with WSJT-X.

