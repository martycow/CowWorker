---
owner: marty
created: "2026-09-13"
last_verified: "2026-09-13"
status: draft
---

# Terminology

These working definitions guide documentation. Unresolved terms do not establish implementation contracts.

| Term | Meaning |
| --- | --- |
| Job Posting | An original advertisement or message that describes a role. |
| Vacancy / Job | Vacancy is the current code/UI name for a saved job opportunity. New requirements also call it Job. |
| Application | A user's attempt to obtain a specific role; its state is separate from the posting state. |
| Career Profile | Confirmed experience, skills, education, and other career facts about the user. |
| Resume | A versioned document that presents the user's qualifications. |
| Cover Letter | A versioned introduction tailored to a specific application. |
| Submitted Copy | The exact document version sent with an application. |
| Next Action | A concrete follow-up step, optionally with a due date. |
| Journey | The career history concept; its exact data boundaries remain open. |
| Match Evidence | Requirements compared with supported profile facts, including gaps and unknowns. |
| Fit Score | An unresolved concept term; no formula or offer-probability claim is approved. |
| BYOK | Bring Your Own Key: access to an AI service through a user-supplied API key. |
| Local-first | Local work remains available without a server connection. |
| ADR | Architecture Decision Record; decisions are grouped in files of up to 25 records. |
| Company | Independent employer identity with many vacancies and explicit user relationships. Planned, not yet a model. |
| AI Operation | Any action that uses a model. It does not imply a charge. |
| AI Attempt | One provider request within a logical AI Operation, including retries. |
| Background Task | A durable local unit of long work, owned by the planned Rust runner. |
| Universal Import | Shared acquisition-to-review pipeline that can accept input before its category is known. |
| Provenance | Source, method, confidence, and time associated with a candidate fact. |
| User Override | An explicit manual value that takes precedence over extracted/generated candidates, including an empty value. |
| Review Draft | Persisted import or change proposal awaiting explicit user decisions. |
| Task Center | Global UI for progress, task details, cancellation, and retry. |

See [the original concept](../CONCEPT.md) and [decision index](../DECISIONS.md).
