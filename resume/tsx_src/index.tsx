import type { ReactNode } from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Work Experience',
    Svg: require('@site/static/img/suitcase.svg').default,
    description: (
      <>
      <hr/>
      <h2>
  <strong>Software Engineer II</strong>
</h2>
<h4>
  <em>Hyster-Yale Materials Handling</em>
</h4>
<ul>
  <li><strong>Employment:</strong> August 2023 - Present</li>
  <li>
    <strong>Accomplishments:</strong>
    <ul>
      <li>Developed embedded C firmware for safety-critical forklift microcontrollers, targeting reliability, deterministic execution, and compliance with industry safety standards.</li><li>Implemented secure boot improvements and contributed to firmware authentication workflows, strengthening protection against tampering and unsafe OTA updates.</li><li>Designed and built a Rust-based CAN logging and analysis tool using C FFI to interface with Vector XL hardware, enabling real-time and offline diagnostic capability.</li><li>Improved internal debugging workflows by creating Python and Rust utilities for log parsing, firmware build artifact automation, and runtime analysis.</li><li>Collaborated with cross-functional electrical, controls, and test engineering teams to deliver robust cyber-physical systems in industrial environments.</li>
    </ul>
  </li>
</ul>
<h2>
  <strong>Data Science Intern</strong>
</h2>
<h4>
  <em>Seasalt.AI</em>
</h4>
<ul>
  <li><strong>Employment:</strong> July 2022 - May 2023</li>
  <li>
    <strong>Accomplishments:</strong>
    <ul>
      <li>Improved Spanish and Bahasa Indonesian Kaldi ASR models by over 40\% WER reduction through acoustic/LM tuning, pipeline optimization, and dataset refinement.</li><li>Automated multilingual data collection and processing using Python and Bash, scraping and cleaning thousands of hours of captioned audio for model training.</li>
    </ul>
  </li>
</ul>
      </>
    ),
  },
  {
    title: 'Education',
    Svg: require('@site/static/img/education.svg').default,
    description: (
      <>
      <hr/>
      <h2>
  <strong>Master of Science in Cybersecurity</strong>
</h2>
<h4>
  <em>Georgia Institute of Technology</em>
</h4>
<ul>
  <li><strong>Attendance:</strong> August 2025 - Present</li>
  <li>
    <strong>Details:</strong>
    <ul>
      <li>Current GPA: 4.00</li><li>Coursework focuses on embedded and systems security including secure system design, binary exploitation, network security, database security, and applied cryptography.</li><li>Exploring resilience and hardening techniques for embedded Linux, IoT firmware, and cyber-physical systems.</li>
    </ul>
  </li>
</ul>
<h2>
  <strong>Bachelor of Science in Computer Science</strong>
</h2>
<h4>
  <em>Washington State University</em>
</h4>
<ul>
  <li><strong>Attendance:</strong> Graduated May 2023</li>
  <li>
    <strong>Details:</strong>
    <ul>
      <li>GPA: 3.9 / 4.00</li><li>Relevant Coursework: Operating Systems, Cryptography, Embedded Systems, Data Structures, Algorithms, Software Engineering, Assembly, Statistics, and Linear Algebra.</li>
    </ul>
  </li>
</ul>
      </>
    ),
  },
  {
    title: 'Personal Projects',
    Svg: require('@site/static/img/memory.svg').default,
    description: (
      <>
        <hr/>
        <h2>CAN Logger</h2>
        <h4>Designed and developed a CAN logger application using the Rust programming language.</h4>
        <ul>
          <li>Displayed live and recorded CAN traffic for real-time debugging and post-analysis.</li>
          <li>Reverse engineered the .blf file format to enable reading and writing industry-standard CAN log files.</li>
          <li>Optimized CAN data processing for high throughput using async programming and efficient Rust techniques.</li>
        </ul>
        <h2>Rust Driver for Vector XL Devices</h2>
        <h4>Developed a Rust driver for Vector XL devices, enabling CAN message transmission and reception via a C foreign function interface.</h4>
        <ul>
          <li>Designed a robust API to streamline communication with Vector XL hardware for automotive integration.</li>
          <li>Implemented functionality to transmit and receive CAN messages with high performance and low latency.</li>
          <li>Employed comprehensive unit testing with mocking and integration tests to ensure reliability and maintainability.</li>
        </ul>
        <h2><a href="https://gitlab.com/wsuv1/cs427_cryptography/rsa_signatures">RSA Signatures</a></h2>
        <h4>Implemented a simplified, highly insecure version of RSA in Rust for digital signatures, featuring two modes: ”sign” and ”verify”.</h4>
        <ul>
          <li>Developed the ”sign” mode to generate random primes, modulus, and totient, and calculate an encryption key. Utilized ELFhash for hashing the message, encrypted the resulting number with the encryption key to generate a digital signature, and verified its integrity.</li>
          <li>In ”verify” mode, the program read inputs and matched signatures to hashes computed from the message, identifying forged messages. Key algorithms used included Miller-Rabin primality test, ElfHash, and exponentiation by squaring.</li>
        </ul>
      </>
    ),
  },
];


function Feature({ title, Svg, description }: FeatureItem) {
  return (
    <div className={clsx('col col--12')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h1">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}