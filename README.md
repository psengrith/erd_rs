# Entity Relation Diagram (ERD) from Rust Struct

<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<center>
      <div><img src="./logo.jpeg" width="169px;" alt="erd_rs"/></div>
</center>
<br/>
<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->


`erd_rs` is command-line utility for creating Entity Relation (ER) Diagram (or class/struct diagram) from codes written in Rust.

Properly documenting entity (or class) is a best practice in software development process. It enhance software maintainability, and allow developers to communicate effectively with team members and other stakeholders. However, most developers prefer to express their solution and idea through codes, instead of drawing. This command line takes Rust code as the source of truth to produce a entity relation diagram markdown file. This would saving their time and energy for more coding.

## Get Started 👍

```bash
# install dependency command
cargo install cargo-expand

# display help message on how to use this command
erd_rs -h 
```

**Example Usage:**

Create mermaid markdown file (i.e., `ER.mmd`) from Rust source code in [`example-project`](./examples/example-project/).

```bash
erd_rs -d ./examples/example-project/
```

*Output:* See [ER.mmd](./examples/example-project/ER.mmd)

*Preview:*

```mermaid
---
title: ER Diagram
---
classDiagram
 GeneModel "n..n" -- "" OrganismModel : exist_in
 class GeneModel {
  -u32 id
  +String name
  +String nucleotide_5_end
  +OrganismModel organism
 }
 GeneModel: +new(id, name, nucleotide_5_end, organism) GeneModel
 GeneModel: +set_id(id) 
 GeneModel: -summarized() String
 GeneModel: ~summarized2() String
 GeneModel: +default() GeneModel
 class OrganismModel {
  -u32 id
  +String nomenclature
 }
 OrganismModel: +default() OrganismModel
 OrganismModel: +new(id, nomenclature) OrganismModel
 OrganismModel: -log_to_console() 
```

## Contributors ✨

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/psengrith"><img src="https://avatars.githubusercontent.com/u/172288069?v=4" width="100px;" alt=""/><br /><sub><b>Sengrith</b></sub></a></td>
     </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->
