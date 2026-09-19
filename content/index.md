---
name: Nathan Spelts
tagline: Embedded software engineer — secure firmware, low-level systems, Rust and C.
description: Nathan Spelts — embedded software engineer. Resume, projects, and writing.
# Linked from the site: no phone, and the email stays spelled out.
public: true
pdf_title: Nathan Spelts' CV
location: Vancouver, WA
email: nathan [at] spelts [dot] net
link: Website | https://nathanspelts.com | pdf
link: GitHub | https://github.com/micro-wizard
link: GitLab | https://gitlab.com/nspelts | web
link: LinkedIn | https://linkedin.com/in/nathanspelts
link: Resume (PDF) | /file/nathanSpeltsResume.pdf | web
---

## Summary

Embedded software engineer specializing in secure firmware, low-level systems, and Rust and C development for cyber-physical systems. Passionate about reliable embedded design, secure boot, and building tools that strengthen the entire development lifecycle.

## Work Experience

### [Hyster-Yale Materials Handling](https://www.hyster-yale.com/)

Software Engineer II · Fairview, OR · Aug 2023 – Present

- Developed embedded C firmware for safety-critical forklift microcontrollers, targeting reliability, deterministic execution, and compliance with industry safety standards.
- Implemented secure boot improvements and contributed to firmware authentication workflows, strengthening protection against tampering and unsafe OTA updates.
- Designed and built a Rust-based CAN logging and analysis tool using C FFI to interface with Vector XL hardware, enabling real-time and offline diagnostic capability.
- Improved internal debugging workflows by creating Python and Rust utilities for log parsing, firmware build artifact automation, and runtime analysis.
- Collaborated with cross-functional electrical, controls, and test engineering teams to deliver robust cyber-physical systems in industrial environments.

### [Seasalt.AI](https://seasalt.ai)

Data Science Intern · Remote · Jul 2022 – May 2023

- Improved Spanish and Bahasa Indonesian Kaldi ASR models by over 40% WER reduction through acoustic/LM tuning, pipeline optimization, and dataset refinement.
- Automated multilingual data collection and processing using Python and Bash, scraping and cleaning thousands of hours of captioned audio for model training.

## Education

### [Georgia Institute of Technology](https://www.gatech.edu/)

Master of Science in Cybersecurity – *Cyber-Physical Systems* · Remote · Aug 2025 – Present

- Current GPA: 4.00
- Coursework focuses on embedded and systems security including secure system design, binary exploitation, network security, database security, and applied cryptography.
- Exploring resilience and hardening techniques for embedded Linux, IoT firmware, and cyber-physical systems.

### [Washington State University](https://wsu.edu/)

Bachelor of Science in Computer Science · Vancouver, WA · Graduated May 2023

- GPA: 3.9 / 4.00
- Relevant Coursework: Operating Systems, Cryptography, Embedded Systems, Data Structures, Algorithms, Software Engineering, Assembly, Statistics, and Linear Algebra.

## Projects {.web}

### CAN Logger

Designed and developed a CAN logger application using the Rust programming language.

- Displayed live and recorded CAN traffic for real-time debugging and post-analysis.
- Reverse engineered the .blf file format to enable reading and writing industry-standard CAN log files.
- Optimized CAN data processing for high throughput using async programming and efficient Rust techniques.

### Rust Driver for Vector XL Devices

Developed a Rust driver for Vector XL devices, enabling CAN message transmission and reception via a C foreign function interface.

- Designed a robust API to streamline communication with Vector XL hardware for automotive integration.
- Implemented functionality to transmit and receive CAN messages with high performance and low latency.
- Employed comprehensive unit testing with mocking and integration tests to ensure reliability and maintainability.

### [RSA Signatures](https://gitlab.com/wsuv1/cs427_cryptography/rsa_signatures)

Implemented a simplified, highly insecure version of RSA in Rust for digital signatures, featuring two modes: "sign" and "verify".

- Developed the "sign" mode to generate random primes, modulus, and totient, and calculate an encryption key. Utilized ELFhash for hashing the message, encrypted the resulting number with the encryption key to generate a digital signature, and verified its integrity.
- In "verify" mode, the program read inputs and matched signatures to hashes computed from the message, identifying forged messages. Key algorithms used included Miller-Rabin primality test, ElfHash, and exponentiation by squaring.

## Skills

- **Programming Languages:** Rust, C, Python, C++, Lua, Bash, Powershell, Kotlin, Swift, Javascript, Java
- **Embedded & Systems:** Secure Boot, Bootloaders, Bare-Metal ARM, RTOS Concepts, GPIO / I2C / SPI / UART / CAN, Memory-Mapped I/O, Firmware Debugging, GDB, QEMU
- **Tools & Technologies:** Git, Docker, Kaldi, FFI (Rust/C), CI/CD Automation, Linux, LaTeX
