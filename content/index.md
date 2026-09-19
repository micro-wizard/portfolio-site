---
name: Nathan Spelts
tagline: Firmware engineer — embedded C, ECU flashing, low-level systems, Rust.
description: Nathan Spelts — embedded software engineer. Resume, projects, and writing.
# Linked from the site: no phone, and the email stays spelled out.
public: true
pdf_title: Nathan Spelts' CV
location: Vancouver, WA
email: nathan at spelts dot net
link: Website | https://nathanspelts.com | pdf
link: GitHub | https://github.com/micro-wizard
link: GitLab | https://gitlab.com/nspelts | web
link: LinkedIn | https://linkedin.com/in/nathanspelts
link: Resume (PDF) | /file/nathanSpeltsResume.pdf | web
---

## Summary

I write embedded C for automotive ECU reprogramming, after nearly three years on safety-critical forklift firmware, secure boot, and Rust tooling for CAN diagnostics. Currently pursuing an M.S. in Cybersecurity at Georgia Tech, focused on cyber-physical systems.

## Work Experience

### [Salvo Software](https://www.salvosoftware.com/)

Firmware Engineer · Vancouver, WA · Apr 2026 – Present

- Maintain production embedded C firmware for an automotive diagnostic tool that reprograms vehicle ECUs.
- Implement flash routines for reprogramming a wide range of ECUs, with a focus on reliability.
- Develop .NET cloud services that support the diagnostic tool.

### [Hyster-Yale Materials Handling](https://www.hyster-yale.com/)

Software Engineer II · Fairview, OR · Aug 2023 – Apr 2026

- Developed safety-critical embedded C firmware for forklift controllers under industry safety standards.
- Improved secure boot and firmware authentication to guard against tampering and unsafe OTA updates.
- Built a Rust CAN logger and analyzer for Vector XL hardware via C FFI, for live and offline diagnostics.

### [Seasalt.AI](https://seasalt.ai)

Data Science Intern · Remote · Jul 2022 – May 2023

- Reduced word error rate over 40% for Spanish and Indonesian Kaldi ASR models through model and data tuning.
- Automated scraping and cleaning of thousands of hours of captioned training audio in Python and Bash.

## Education

### [Georgia Institute of Technology](https://www.gatech.edu/)

Master of Science in Cybersecurity – *Cyber-Physical Systems* · Remote · Aug 2025 – Present

- Current GPA: 4.00
- Coursework: secure system design, binary exploitation, network and database security, applied cryptography.

### [Washington State University](https://wsu.edu/)

Bachelor of Science in Computer Science · Vancouver, WA · Graduated May 2023

- GPA: 3.9 / 4.00

## Projects

### [Raspberry Pi 4 Bare-Metal Kernel](https://github.com/micro-wizard/raspi4_rust_bootloader)

A `no_std` Rust kernel for the Raspberry Pi 4, booted straight from the SD card.

- Wrote AArch64 startup assembly that parks secondary cores, zeroes BSS and sets up the stack for Rust.
- Drove the VideoCore GPU via its mailbox interface for a double-buffered framebuffer with text rendering.
- Unit-tested the hardware-facing code on the host against a mocked mailbox, run in GitHub Actions CI.

### [Particle Simulator](/particles/) {.web}

A falling-sand physics sandbox in JavaScript and WebGL. Try it on the [Particles](/particles/) page.

- Simulates sand, water, lava, steam, fire and growing seeds with local density and reaction rules.
- Renders every particle with a small WebGL sprite renderer, targeting 60 fps.

### CAN Logger {.web}

Designed and developed a CAN logger application using the Rust programming language.

- Displayed live and recorded CAN traffic for real-time debugging and post-analysis.
- Reverse engineered the .blf file format to enable reading and writing industry-standard CAN log files.
- Optimized CAN data processing for high throughput using async programming and efficient Rust techniques.

### Rust Driver for Vector XL Devices {.web}

Developed a Rust driver for Vector XL devices, enabling CAN message transmission and reception via a C foreign function interface.

- Designed a robust API to streamline communication with Vector XL hardware for automotive integration.
- Implemented functionality to transmit and receive CAN messages with high performance and low latency.
- Employed comprehensive unit testing with mocking and integration tests to ensure reliability and maintainability.

## Skills

- **Programming Languages:** C, C++, Rust, Python, C#, Bash
- **Embedded & Systems:** Secure Boot, Bootloaders, Bare-Metal ARM, RTOS, I2C / SPI / UART / CAN, Memory-Mapped I/O, GDB, QEMU
- **Tools & Technologies:** .NET, Git, GitHub Actions, Docker, Linux, FFI (Rust/C), Vector XL
