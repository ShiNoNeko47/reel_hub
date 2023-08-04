#! /usr/bin/env python3
import colorsys
import time


def setup():
    print(
        f"css;messagedialog button, window button {{transition: border-image .5s, background .5s;}}",
        flush=True,
    )


def main():
    setup()
    """
     - if you want to send data without waiting for reel_hub to send a request
       first you have to use flush=True when printing

    css:
         - css has to be written in single line to be sent and read at once
         - use semicolon (;) where you normally would in css

        format:
            "css" -> tells reel_hub to use the following data as css
            css -> everything after the first semicolon (;)
    """

    while True:
        num_steps = 20
        hue = 0.0
        step_val = 1.0 / num_steps
        for _ in range(num_steps):
            rgb = colorsys.hsv_to_rgb(hue, 1, 1)
            hue += step_val
            hue %= 1.0
            r = round(rgb[0] * 255)
            g = round(rgb[1] * 255)
            b = round(rgb[2] * 255)
            rgb_ints = (
                hex(r).replace("0x", ""),
                hex(g).replace("0x", ""),
                hex(b).replace("0x", ""),
            )
            try:
                print(
                    f"css;@define-color accent #{rgb_ints[0]:02}{rgb_ints[1]:02}{rgb_ints[2]:02};",
                    flush=True,
                )
            except BrokenPipeError:
                exit(0)

            time.sleep(0.5)


if __name__ == "__main__":
    main()
