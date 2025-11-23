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
          <strong>Embedded Software Engineer</strong>
        </h2>
        <h4>
          <em>Hyster-Yale Materials Handling</em>
        </h4>
        <ul>
          <li><strong>Employment Length:</strong> August 1st, 2023 - Present</li>
          <li>
            <strong>Responsibilities:</strong>
            <ul>
              <li>Developed and maintained embedded C solutions for forklift microcontrollers, ensuring performance, reliability, and safety in industrial applications.</li>
              <li>Collaborated with a cross-functional team of engineers, testers, and managers to deliver secure and innovative embedded software solutions.</li>
              <li>Contributed to technical documentation and quality assurance processes, enhancing the development lifecycle and ensuring compliance with industry standards.</li>
            </ul>
          </li>
        </ul>
        <hr/>
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
        <hr/>
        <h2>Rust Driver for Vector XL Devices</h2>
        <h4>Developed a Rust driver for Vector XL devices, enabling CAN message transmission and reception via a C foreign function interface.</h4>
        <ul>
          <li>Designed a robust API to streamline communication with Vector XL hardware for automotive integration.</li>
          <li>Implemented functionality to transmit and receive CAN messages with high performance and low latency.</li>
          <li>Employed comprehensive unit testing with mocking and integration tests to ensure reliability and maintainability.</li>
        </ul>
        <hr/>
        <h2><a href="https://gitlab.com/wsuv1/cs427_cryptography/rsa_signatures">RSA Signatures</a></h2>
        <h4>Implemented a simplified, highly insecure version of RSA in Rust for digital signatures, featuring two modes: ”sign” and ”verify”.</h4>
        <ul>
          <li>Developed the ”sign” mode to generate random primes, modulus, and totient, and calculate an encryption key. Utilized ELFhash for hashing the message, encrypted the resulting number with the encryption key to generate a digital signature, and verified its integrity.</li>
          <li>In ”verify” mode, the program read inputs and matched signatures to hashes computed from the message, identifying forged messages. Key algorithms used included Miller-Rabin primality test, ElfHash, and exponentiation by squaring.</li>
        </ul>
        <hr/>
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
          Bachelor&apos;s of Science in Computer Science
        </h2>
        <h4>
          <em>Washington State University</em>
        </h4>
        <ul>
          <li><strong>GPA:</strong> 3.89 / 4.00</li>
          <li><strong>Coursework:</strong> Functional Programming | Object Oriented Programming | Statistics | Linear Algebra | Calculus |
          Assembly | Operating Systems | Data Structures | Databases | Software Engineering | Cryptography | Algorithms</li>
          <li><strong>References:</strong> Dr. Grant Williams - <a href="mailto:grant.s.williams@wsu.edu">grant.s.williams@wsu.edu</a></li>
        </ul>
        <hr/>
        <h2>Master&apos;s in Physical Cyber-Security</h2>
        <h4><em>Actively applying to schools</em></h4>
        <ul>
          <li><strong>Goal:</strong> Improve embedded system security skills.</li>
        </ul>
        <hr/>
      </>
    ),
  },
];


function Feature({ title, Svg, description }: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
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
