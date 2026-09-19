---
title: Creating Videos from Reddit Posts and Comments
slug: youtube_content_creator
summary: A Deep Dive into an AI-Powered Project
---

## Introduction
In this blog post, we will explore an intriguing project that utilizes AI and various libraries to create engaging videos from Reddit posts and comments. The project's main goal is to transform text-based content into visually appealing videos by combining Playwright for web scraping, moviepy for video editing, and SQLite for data management. Let's dive into the details and discover what makes this project interesting and valuable.

[Source code can be found here.](https://github.com/FashionablyNate/yt_content_creator)
 
## Project Overview
The main.py file serves as the entry point for the project. It imports necessary libraries and modules, such as dataclasses, os, asyncio, knapsack, shutil, and several custom modules. The script then defines a Content data class and initializes a content_dict dictionary to store content information.

The project's workflow starts by connecting to the Reddit API using the Reddit wrapper module. It retrieves posts from a specific subreddit and iterates through each post. If a video for a particular post doesn't already exist, the script proceeds to create content for that post.

The create_content_post() function generates audio and captures a screenshot of the post using the take_screenshot_post() function. The resulting content is stored as a Content object in the content_dict dictionary.

If the post's audio duration is longer than 60 seconds, the script inserts the video into a database and removes the generated content directories. Otherwise, it proceeds to process the comments associated with the post.

For each comment, the script creates content using the create_content_comment() function, which generates audio and captures a screenshot of the comment using the take_screenshot_comment() function. The resulting content is stored in the content_dict dictionary, along with duration, upvotes, and comment IDs.

To optimize the selection of comments, the script uses the knapsack algorithm to find the best combination of comments based on duration and upvote count. Once the selection is made, it creates a video by combining the post content and selected comments using the create_video() function.

The video is then uploaded to YouTube using the get_authenticated_service() and initialize_upload() functions from the upload_video module. Finally, the generated video is saved to the output_video.mp4 file.

Other modules in the project include take_screenshot.py, which uses the Playwright library to capture screenshots of posts and comments on Reddit, and create_video.py, which handles the video creation process, including audio synthesis and video editing.

## Key Features and Technologies:

- Web Scraping with Playwright: The project leverages Playwright to scrape Reddit posts and comments and capture their visual representation.
- Text-to-Speech (TTS) Synthesis: The TTS library is used to convert text content into synthesized audio, enhancing the user experience when watching the videos.
- Video Editing with Moviepy: Moviepy, a Python library for video editing, enables the combination of audio, images, and video clips to create compelling and dynamic video content.
- Knapsack Algorithm: The knapsack algorithm is employed to select the most suitable comments based on their duration and upvote count, ensuring an optimal combination for the final video.
- SQLite Database: The project utilizes an SQLite database to store video IDs, ensuring that duplicate videos are not generated for the same Reddit post.

## Conclusion:
The project presented here showcases the integration of various technologies and libraries to automate the process of transforming text-based Reddit content into visually engaging videos. By leveraging AI, web scraping, text-to-speech synthesis, video editing, and database management, the project demonstrates the power of Python for content creation and automation.

The ability to convert text content into multimedia videos not only enhances the user experience but also opens up opportunities for content creators to repurpose existing textual content and reach a wider audience through visual storytelling.

With further refinement and customization, this project could be expanded to support different platforms, integrate additional AI capabilities such as sentiment analysis or image recognition, and provide more advanced video editing features.

One interesting aspect of this project is the combination of different libraries and tools to achieve the desired outcome. Playwright, a powerful web scraping library, enables the script to interact with Reddit's web interface and capture screenshots of posts and comments. By leveraging Playwright's capabilities, the project extracts the visual content necessary for video creation.

The integration of TTS technology adds another layer of engagement to the videos. By synthesizing the text content into audio, the project brings the posts and comments to life, enabling users to consume the content in a more immersive way. This demonstrates the potential of TTS in enhancing multimedia experiences and making content more accessible to a broader audience.

Moviepy, a versatile video editing library, plays a crucial role in combining audio, images, and video clips to create the final videos. It provides a user-friendly and intuitive interface for video composition, allowing for seamless integration of different media elements. The project showcases how Moviepy can be utilized to automate the video editing process and produce professional-looking videos.

The implementation of the knapsack algorithm introduces an element of optimization to the project. By selecting the most suitable comments based on their duration and upvote count, the algorithm ensures that the final video maintains an optimal length and includes the most relevant and popular comments. This demonstrates how algorithmic techniques can be applied to content curation and enhance the quality of the generated videos.

Furthermore, the use of an SQLite database for video ID management adds a layer of data persistence and ensures that duplicate videos are not generated for the same Reddit post. This highlights the importance of data management and provides a foundation for scalability and future enhancements, such as tracking video analytics or managing user preferences.

Overall, this project showcases the power of Python and its ecosystem of libraries and tools in automating content creation and transforming text-based content into visually appealing videos. By leveraging AI, web scraping, text-to-speech synthesis, video editing, and database management, the project opens up possibilities for innovative content generation and storytelling.