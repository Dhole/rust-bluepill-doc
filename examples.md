# Components

Mosfet transistors can be easily used as a digital switch.

A darlington transistor contains two transistors allowing higher current
amplification than a single transitor.

## IRF520 Mosfet

![](/examples/irf520.jpg)

## 30N06L Mosfet

![](/examples/30n06l.jpg)

## BC517 Darlington Transistor

![](/examples/bc517.jpg)

## BD139 Transistor

![](/examples/bd139.jpg)

# Circuits

## Speaker

The following circuit is required for code [app2](/app2).

![](/examples/speaker.jpg)

The following calculation is an example to figure out the resistor value that
goes with the speaker.

![](/examples/speaker_values.jpg)

## Potentiometer

The following circuit is required for code
[potentiometer](app/examples/potentiometer.rs) and
[potentiometer2](app/examples/potentiometer2.rs)

![](/examples/potentiometer.jpg)

## Display

ALERT: The rotating logo code is not very clean.

The following connections are required for code
[display](app/examples/display.rs),
[display2](app/examples/display2.rs)
[rotating logo](https://github.com/Dhole/ssd1306/blob/master/examples/image_i2c.rs)

```
GND -> GND
VCC -> +5V
PB9 -> SDA
PB8 -> SCK

PB5 -> +---+
       | O |
GND -> +---+
```

About pull-up and pull-down:
![](examples/pullup-pulldown.png)
