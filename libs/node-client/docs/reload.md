```mermaid
flowchart TB
  A(Start) --> B[acquire mutex lock]
  B --> C[retrieve new events]
  C --> D{{digest}}
  D --> E[release mutex lock]
  E --> F(Stop)
```
