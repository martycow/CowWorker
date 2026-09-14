---
owner: marty
created: "2026-09-12"
last_verified: "2026-09-13"
status: historical-concept
---

# Initial Concept

This document preserves the original brainstorm, including unresolved and superseded ideas.
Use [the decision index](DECISIONS.md), [current stack](tech/STACK.md), and [UI direction](design/UI.md) for accepted choices.
The review date confirms this document's role; it does not verify hardware, services, or brainstorm claims.

This document stores a concept for a project. Here are my thoughts, ideas, desires and a vision for the future product.

## Name

`CowWorker`
It sounds funny and correlates with Moo.exe

## Main Purpose

- To find a job! I am the target audience right now.
- To learn new skills, improve my knowledge, and experiment.
- To include it in my devlog and attract attention that way. Maybe I can even make a video about it!
- Improve my networking skills.
- Manage different job-finding platforms in one place.
- Come up with the most interesting, unique, attractive and fun ideas to implement. Treat the development process as a jorney.
- Have an AI-first development experience.
- Have fun!

## What this project is

CowWorker is a multiplatform app that helps users with the job hunting process. Users can manually or automatically add job vacancy descriptions (plain text, links, and screenshots). The app uses AI (BYOT) to boost productivity, help create and adjust resumes, write tailored cover letters, and manage applied vacancies. The app tracks the status of added job postings, displays a fit score, and shows job posting statistics, as well as information about the company and its employees. The app also collects statistics, performs analyses, and displays research results with graphs and charts. It also stores the history of user's employment including jobs which were successfully found via CowWorker. It has it's own knowledge base with articles, notes, cheetsheets and vocabulary to prepare for interviews.

## Usage Scope

CowWorker is intended to be used as an assistant during job hunting including all the steps. It also stores and updates info for current job and side projects. CowWorker is just a set of tools for EVERYTHING related to work and personal career building.

## Requirements

- Must support Windows, MacOS, iPhone, iPad;
- Would be nice to support Busy Bar through USB and Wi-Fi to display some important information or notifications;
- Convinient, flexible, adaptive modern UI;
- Fast and lightweight;
- Can do some work automatically on a background or on schedule;
- Preferably connecting new job-searching platforms should be as easy as possible;
- Auth using e-mail, LinkedIn, Telegram;
- All the data is stored in database;
- Desktop apps must have an installer with ability to being updated;

## Tech Stack

- C# or Rust for backend
- Angular or React for frontend
- Preferably C++ and Qt may be used for frontend if it's worth it
- SQLite or other DB

## Supported Platforms

- Windows 11 desktop app is a MUST
- MacOS Mini desktop app is a MUST
- iPhone and iPad apps are desired but not the top-priority

## Available Equipment

- Asus Strix G614JV (Windows 11, 32Gb RAM)
- Mac Mini M6 (macOS Golden Gate, 32Gb RAM)
- Raspberry PI 4 (Ubuntu, 8Gb RAM)
- Freenove Complete Starter Kit
- Busy Bar
- TourBox Neo

## Environment

- The domain for CowWorker is bought and operated on Cloudflare
[Domain](https://cowworker.com)
- The server is gonna be running on Raspberry PI;
- May use Claude Platform API if needed;
- May use OpenAI Platform API if needed;
- May use LinkedIn API if needed;
- Telegram Bot `@cowworker_bot` is used to notify user;
- UptimeRobot is used to check server's status;
- Resend is used for sending e-mails;
- There must be Dev, Test and Live environments;
- Busy Bar is accessible though USB (10.0.4.20) and Wi-Fi (10.0.0.120)

## AI

- As a default option, app should look for installed Claude Code, Codex or some other AI tools, and use it to launch AI locally inside of an app.

- There should be an option to use API key (BYOK - Bring Your Own Key). However, it needs a research because I'm not sure how it would work on iPhone/iPad.

- User can use Claude, Codex and may have some additional help from others (like Grok).

## UI

- Modern, minimalistic;
- A little bit fun. It's COWworker, you know!
- A cow worker on logo;
- Light/Dark theme switch;
- Customizable dashboard
- Appearance Settings
- General Settings
- Graphs, Data Visualization, Enjoyable Sheets

## Primary Terms

1) Job Posting - defines source of job. Vacancy, some post from social media. Something that described work conditions requirements, salary etc
2) Application - User's attempt to apply for a job for a specific company at a specific position. It means user made a decision to apply for a provided job.
3) Resume - Description of user's work experience, contact information, education, skills, known languages and tech stack. It's rapidly edited, meaning it may have many different versions.
4) Cover Letter - An introduction letter which is used as a first contact message to an employer. Usually it's tailored for a specific employer and should sound frienly/professional depending 
5) Fit Score - A metric which is used to check the possibility of getting a job offer in case of appliance to this job position. It also considers different factors: reviews, results of deep research (web-search + reasoning). Fit Score is checked right after getting a job posting. One calculation is AI operation. It spends tokens. This is UNCLEAR, but there are two ways: sell tokens with 3% fee or sell packs of credits for different price.

## Features brainstorm
1) Modular flexible architecture from the beginning
2) New features added as new modules
3) There are only a few potential Core modules which are mandatory.
4) "Jorney". This word is used as a synonym for a project. Jorney is a user's Lifetime: information about user, employment status, employment history, education, skills, certificates, licences, allowance documents, signed job offers and contract terminations, W-2, I-1099, LLC, everything that is related to user's skills, possibilities, profession, career, income, mental state, benefits, attened cources, conferences, showcases
5) Technically, Jorney must be as flexible as possible. It is Timeline in it's form. There are Events which may randomly appear. Event types examples: lay off, paid leave, resignation, dismissal.
6) User can import files with extentions: .pdf, .docx, .doc, .md, .txt, .xls
7) Imported files are parsed, analyzed and then translated into Application-friendly format
8) Possibly application should use different social media and jobhunting portals APIs as much as possible. Update profiles, keep updated to what happens there relatable to jobhunting process.
9) Deep AI integration using Anthropic's and OpenAI's AI as a default. It is possible to add support of local models
10) Notifications and Calendar management. Consider that user may be using Windows, macOS and iOS (and Android later) simultaneously. Jobhunting-related events in Apple Calendar and Google Calendar should be synced.
11) User can see and manage the list of all the data in a convinient way
12) For now only English support, but I need to think what languages to support next
13) AI-Traslation support
14) AI model selection for each AI operation 15) Context memory must be stored in Project's folder
16) All operations only able with change of related context file. All context files are predefined, but it's unclear
17) Quizes, optional quationaries, "Tell me about your last job" type of input window. Using small optional dialogs to refresh current status of user. It shouldn't be annoying
18) ADHD-friendly!!! Every step is WOW-WOW! "Congratulations for fill up a form and checked a checkbox, you're the greatest human on Earth! Oh and here are fireworks by the way in your honour."
19) Tech Stack Glossary. It's a pipeline. Once in a while AI model makes a deep research finding trends, new trendy libraries, technologies, startups, some terms. Then it goes to another agent which creates, updates and mainates the list of Glossary. Then another agent analyzes data in Glossary and creates a list for rendering. It includes pictures and icons. It's like tags. Also User can see the trending graphs
20) Every mention of existing term from Glossary is underscored and may be clickable to see the definition. 
21) Different writers for different scenarious
22) Salary Negotiator using AI agent and skills
23) Career Developer using AI agent and skills
24) Cover letter editor
25) Resume editor has a canvas. User can make changes to this canvas, can undo/redo, each significant change is stored as a new version of resume.
