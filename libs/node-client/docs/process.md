```mermaid
flowchart TB
  A(Start) --> B[acquire mutex lock]
  B --> C{{digest}}
  C --> D[execute command]
  D --> E[[verify business invariants]]
```
