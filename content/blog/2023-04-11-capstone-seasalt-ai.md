---
title: ASR Capstone at Seasalt AI
slug: capstone_seasalt_ai
summary: Machine learning project that performs automated speech recognition on Indonesian and Spanish.
---

It’s been nearly a year since my last post, but I’m excited to share details about my latest project! As I approach the end of my senior year at WSU, I’ve spent the entire year working on a capstone project in collaboration with [Seasalt AI](https://seasalt.ai/). Our work has focused on two tasks: developing an Indonesian model from scratch during the first semester, and retraining Seasalt AI’s existing Spanish model this semester.

## Indonesian Model
The process of building a model from scratch began with locating an open-source corpus containing segmented audio files and time-stamped captions. We utilized this corpus to train our initial model using [Kaldi](https://kaldi-asr.org/), an open-source ASR project. The training pipeline involves normalizing the data, extracting mel frequency cepstral coefficients (MFCC) to facilitate approximation and save space, and generating statistical models to make assumptions about the data. Subsequently, the data undergoes monophone training, which examines individual phonemes, followed by triphone training that considers neighboring phonemes for context. Additional steps include speaker normalization, adapting the model to a specific individual’s voice, and reducing the impact of background noise. Lastly, the model is fine-tuned via time-delayed neural network training, resulting in a baseline Indonesian speech-to-text model.

## Spanish Model

### Data Crawling
For the Spanish model, we began with a provided baseline model. Our primary task was to gather data and use it to retrain this baseline model. We downloaded thousands of hours of YouTube audio and captions using the youtube-dl Python library, with our chosen keywords guiding the crawling process. We developed a script that took these keywords, located videos with Spanish captions, and downloaded all videos from channels containing those keywords. This data was then converted into a Kaldi-compatible dataset.

### Data Validation
We proceeded to validate the data using our model. This involved creating smaller datasets, each containing around 50 hours of audio, and employing the model to confirm the presence of actual Spanish audio. The model assessed the word error rate between the audio and existing captions. The outcome was a condensed list of segments and Kaldi data suitable for retraining the model.

### Model Retraining
Finally, we used the validated data to retrain the baseline model. This relatively straightforward process involved running scripts to enhance the model using the validated data.

## Conclusion
This project has been an invaluable learning experience in machine learning, speech recognition, and working within a corporate environment. It has significantly contributed to my growth as a programmer, honing my skills in Python, Bash, and command-line operations.