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
  (((jobs))),
  (((education))),
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
