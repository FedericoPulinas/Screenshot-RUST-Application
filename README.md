# Rust

# PROGETTO SCREENSHOT

Using the Rust programming language, create a screen grabbing utility capable of
acquiring what is currently shown in a display, post-process it and make it available
in one or more formats.
The application should fulfill the following requirements:

### 1. Platform Support:
The utility should be compatible with multiple desktop
operating systems, including Windows, macOS, and Linux.
### 2. User Interface (UI):
The utility should have an intuitive and user-friendly
interface that allows users to easily navigate through the application's
features.
### 3. Selection Options:
The utility should allow the user to restrict the grabbed
image to a custom area selected with a click and drag motion. The selected
area may be further adjusted with subsequent interactions.
### 4. Hotkey Support:
The utility should support customizable hotkeys for quick
   screen grabbing. Users should be able to set up their preferred shortcut keys.
### 5. Output Format:
The utility should support multiple output formats including
.png, .jpg, .gif. It should also support copying the screen grab to the clipboard.

## As a bonus, the application may also provide the following features:

### 6. Annotation Tools:
The utility should have built-in annotation tools like
shapes, arrows, text, and a color picker for highlighting or redacting parts of
   the screen grab.
### 7. Delay Timer:
The utility should support a delay timer function, allowing users
to set up a screen grab after a specified delay.
### 8. Save Options:
The utility should allow users to specify the default save
location for screen grabs. It should also support automatic saving with
predefined naming conventions.
### 9. Multi-monitor Support:
The utility should be able to recognize and handle
multiple monitors independently, allowing users to grab screens from any of the connected
displays.

***

# Librerire
Aggiungete qua le librerie usate o quelle che possono sembrare utili

**Controllare con quasi SO sono compatibili**

|  Nome  | link                           | Utilità                   | Compatibilità       |
|:------:|:-------------------------------|:--------------------------|:--------------------|
| winit  | https://crates.io/crates/winit | Gestione Input e grafica  | Windows, MacOS, iOS |
