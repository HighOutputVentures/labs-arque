```mermaid
flowchart TB
  A(Start) --> B[acquire mutex lock]
  B --> C{{digest}}
  C --> D[release mutex lock]
  D --> E(Stop)
```
