# Android IME support for Rust

This is a highly experimental PoC project to provide
Android IME support in Rust without any Java-side dependencies.

## Problem

Rust already has many GUI frameworks that can be used on Android,
but all of them lack IME support on Android.

IME stands for Input Method Editor.
It is a software keyboard that allows users to enter text, emojis, suggestions, etc.

The main problem with IME in Android - is that you cannot interact with it without
Java/Kotlin code. And this limitation is built into the framework.

Even if you manage to show the soft keyboard, it will only propagate some KeyEvents
to the View, and even that depends on the IME installed on the device. KeyEvent
itself can only transfer physical keys information and modifiers, meaning there
cannot be any support for the Unicode non-latin symbols, suggestions, etc.

## Any solutions?

One of the possible solutions is to require users to add a Java/Kotlin library as
a dependency and use JNI to interact with it from the native layer. It works, but
it is not very convenient and can lead to de-sync between the native and Java versions.

So the main problem is this Java code part...

What if... we could provide this Java code as part of the Rust library?

This is what this project is about.

## General idea

So the idea is pretty simple:
1. write a small Java library that has all the glue code we need
2. this library must have no dependencies on any other libraries (except android.jar)
3. compile this Java library to a `jar` file
4. convert this `jar` file to a `dex` file that Android can understand
5. embed this `dex` file into out Rust library (include_bytes! is enough)
6. using JNI, create instance of Android's `InMemoryDexLoader` with our `dex` bytes
7. now we can load all classes from the Java library in our Rust library

After this, we have a full communication channel between Rust and Java codebases.

# Project structure

- `android` - Android library with the `InputConnection` glue
- `android-ime` - Rust library that provides a rust-friendly API
- `android-ime-test` - egui-based test application that shows how to use Android IME

# License

MIT or Apache 2.0, Whatever you prefer.
