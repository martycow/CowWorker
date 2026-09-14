---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: current-requirements
---

# Product

This file owns current product requirements. Requirements describe the target, not completed features.
[Architecture](../tech/ARCHITECTURE.md) owns technical boundaries. The [development plan](../management/V0.2.0_DEVELOPMENT_PLAN.md) owns implementation steps and repository evidence.
The [original concept](../CONCEPT.md) remains historical.

## Purpose and baseline

CowWorker connects vacancies, career documents, applications, interviews, offers, and career development in one local workspace.
Marty is the first target user. The interface language is English. Windows 11 and macOS are desktop targets.
Rust, React, TypeScript, Tauri, SQLite, and the selected visual direction remain accepted choices.

The code supports manual vacancy capture, immutable text document versions, application history, next actions, and profile facts.
There is one application per vacancy. Submitted applications retain exact document versions.
The desktop code also supports Company records, durable tasks, reviewed imports, PDF/DOCX text conversion, and a central AI runtime.
The [implementation record](../management/V0.2.0_IMPLEMENTATION.md) identifies the remaining requirements. These additions do not complete every requirement below.
[Verification](../tech/VERIFICATION.md) separates historical Windows results from current evidence.

## Contextual AI

AI is an interface layer. A global panel opens on the right and supports bottom placement.
It follows the active page, entity, document version, and selected field without starting model calls on navigation.
Inline hints, field comments, change proposals, insights, contextual actions, small dialogs, and analysis explanations complement this panel.
Examples include resume bullet improvements, experience suggestions, vacancy extraction explanations, and company insights.

AI must be available everywhere without filling every screen with messages.
Hints remain restrained and dismissible. Results show sources, uncertainty, and missing evidence.
A separate AI Chat page is not the primary workflow. AI cannot invent career facts or silently apply changes.

## Universal Import

Universal Add accepts URL, text, paste, file selection, screenshot/image, and drag-and-drop.
Jobs accept PDF and DOCX as well as text and images. Documents accept PDF, DOCX, text/Markdown, images, and paste.
The user does not need to choose the correct entity type first.

The pipeline is Input → Extraction → Classification → Parsing → Normalization → Optional AI enrichment → Review → Save.
Classification returns a suggested entity/category and confidence. Unknown remains valid. Users can correct classification before or after save.
Document categories retain all existing kinds and add Job Description, Portfolio, Reference, and Unknown.
Resume, Cover Letter, Note, Job Offer, Agreement, Tax Related, and Other remain distinct.

Job review covers company, title, location, salary, employment type, work mode, description, requirements, responsibilities, stack, benefits, source, URL, and dates.
Existing review state, notes, and application history remain intact. Missing values remain unknown.
An imported Job Description can remain a document or create a vacancy through explicit review.

Original inputs and source metadata must support reprocessing.
Extracted values and generated proposals remain separate from user overrides. An explicit empty user value also takes priority.
Refresh, retries, and late results must preserve manual edits and exact submitted documents.

## AI Operation and AI Usage

Every model action uses one consistent AI Operation marker, such as ✦ Analyze job or ✦ Improve bullet.
The marker means that a model is used. It does not mean that the action has a price.
Action details show purpose, provider/model, estimated tokens and cost when available, and an optional credit balance.
Unknown estimates remain unknown. The design supports local models, BYOK, free models, and future subscription credits without requiring billing infrastructure.

AI Usage is a separate destination with summary cards, tables, charts, and operation history.
It shows operation counts, input/output/total tokens, estimated and actual cost, and breakdowns by time, model, provider, and feature.
Feature categories include job analysis, resume tailoring, company research, cover letter generation, document classification, and Other.
Entity filters cover jobs and companies where available. A Project entity is not required solely for this report.
All features use one usage ledger. Failed and cancelled calls can still incur usage.

## Company Hub

Company is independent of Vacancy. One company can have many vacancies with different application stages.
Companies can originate from manual creation, saved/imported vacancies, applications, interviews, offers, and past or current employment.
Past/current employment receives a distinct visual marker. Saving a vacancy does not imply employment.

Cards show logo or fallback, name, description/industry, vacancy counts, application stage summaries, relationships, and useful badges.
Company Details covers identity, website, headquarters, remote policy, founding date, employee count, stack, contact information, social links, and leadership.
It also supports images, related vacancies/applications, notes, research, and employment history.
Missing facts do not block local use. Sources and update dates accompany researched facts.
Refresh preserves user edits. Ambiguous company matches require review instead of automatic merging.

## Global tasks and navigation

The shell contains navigation, the main workspace, contextual AI, Universal Add, and a global Task Center.
Five imported vacancies must progress independently. Navigation must not stop processing or hide its status.
Tasks expose status, stage, progress, elapsed time, details, cancellation, and retry. Estimates appear only when defensible.
A compact indicator opens the Task Center. Notifications remain quiet unless attention is useful.

The planned queue persists across restarts and resumes safe work when the application runs again.
Execution after application exit is outside this delivery scope.
Today, Vacancies, Applications, Documents, Profile, and Settings remain useful destinations.
Companies and AI Usage are additions. Dashboard, Contacts, and generic Analytics are not mandatory new pages.
The entity detail pane remains separate from the global AI panel.

## Delivery boundaries

Model calls and web research require explicit authorization for the data sent. CowWorker does not automatically contact employers.
Synchronization, mobile delivery, multiple applications per vacancy, payments, and an OS background service remain separate work.
The [plan](../management/V0.2.0_DEVELOPMENT_PLAN.md) preserves existing task IDs and includes migration, recovery, and native verification gates.
