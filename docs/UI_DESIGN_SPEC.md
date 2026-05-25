# UI Design Specification

This document defines the visual design system for Meow WCH ISP. It is written
as an implementation contract for Slint UI, AI iteration, and Computer Use QA.

The product is a professional flashing and configuration tool. The UI should
feel precise, calm, technical and trustworthy. It must not look like a toy,
landing page, or board-specific helper.

## Design Principles

- Capability-driven: the UI changes according to the connected device and its
  supported operations.
- Plan before apply: destructive operations are visually separated from read-only
  inspection and always show a plan/diff first.
- Dense but breathable: this is an engineering tool, so information density is
  welcome, but spacing and hierarchy must make risk easy to see.
- No mystery disabled states: every disabled action has a visible reason through
  inline text, tooltip, or side detail.
- One primary task per screen: Dashboard orients, Flash flashes, Memory handles
  regions, Config handles registers, Project runs intent files, Catalog browses.

## App Shell

Default desktop window:

- initial size: `1040 x 720`
- minimum size: `880 x 620`
- preferred max content width: none; the app should use available width
- corner radius: native window chrome only; in-app cards max `8px`
- root background: `#F6F7F8`
- content background: `#FFFFFF`
- sidebar width: `216px`
- top bar height: `56px`
- footer/status bar height: `36px`
- global padding: `16px`
- section spacing: `16px`
- panel spacing: `12px`
- component radius: `6px` or `8px`, never pill-shaped except small status chips

Layout:

```text
+---------------------------------------------------------------+
| Top bar: device state, transport, operation status, app tools |
+---------------+-----------------------------------------------+
| Sidebar       | Page content                                  |
| navigation    | capability-driven panels                       |
|               |                                               |
+---------------+-----------------------------------------------+
| Status/event bar: last event, JSON copy, safety state          |
+---------------------------------------------------------------+
```

## Typography

Fonts:

- macOS UI: `SF Pro Text`, fallback `Helvetica Neue`
- Linux UI: `Inter`, fallback `Noto Sans`, fallback `DejaVu Sans`
- monospace: `SF Mono`, fallback `JetBrains Mono`, fallback `monospace`
- Chinese fallback: `PingFang SC` on macOS, `Noto Sans CJK SC` on Linux

Sizes:

| Token | Size | Weight | Use |
| --- | ---: | ---: | --- |
| `display` | 24px | 700 | page title only |
| `title` | 18px | 700 | panel title |
| `subtitle` | 14px | 600 | section labels |
| `body` | 13px | 400 | normal copy |
| `body-strong` | 13px | 600 | values and labels |
| `caption` | 11px | 500 | chips, metadata |
| `mono` | 12px | 500 | addresses, hex, JSON |

Line heights:

- display: `32px`
- title: `24px`
- body: `20px`
- caption: `16px`
- mono: `18px`

Rules:

- letter spacing is always `0`
- never scale font size with viewport width
- long identifiers use elision in compact cells and full text in detail panes
- hex and register values always use monospace

## Color System

Base:

| Token | Color | Use |
| --- | --- | --- |
| `bg` | `#F6F7F8` | root |
| `surface` | `#FFFFFF` | main panels |
| `surface-muted` | `#F0F2F4` | inactive surfaces |
| `border` | `#D8DEE4` | panel borders |
| `border-strong` | `#B8C0CC` | focused or selected |
| `text` | `#1F2328` | primary text |
| `text-muted` | `#656D76` | secondary text |
| `text-faint` | `#8C959F` | disabled metadata |

Semantic:

| Token | Color | Use |
| --- | --- | --- |
| `blue` | `#2563EB` | selected navigation, read actions |
| `green` | `#1F883D` | connected, verified, success |
| `amber` | `#B7791F` | warning, plan pending |
| `red` | `#D1242F` | destructive, failed, high risk |
| `purple` | `#8250DF` | AI/automation |
| `cyan` | `#0E7490` | transport and catalog |

Tints:

- success bg: `#EAF7EE`
- warning bg: `#FFF4D6`
- danger bg: `#FFEBE9`
- info bg: `#EAF2FF`
- automation bg: `#F2ECFF`

Rules:

- The palette must not become one-note beige, purple, or blue.
- Dangerous controls use red only at the final apply step. Planning and preview
  use amber.
- Disabled destructive buttons use neutral gray, not faded red.

## Icons

Use symbolic icon slots; map them to Slint assets or icon font later.

Required icons:

- device: chip
- transport USB: usb
- transport serial: cable
- scan: radar
- flash: lightning
- verify: check-circle
- erase: trash or eraser
- memory: database
- config: sliders
- project: file-cog
- catalog: list-tree
- automation: bot
- warning: triangle-alert
- danger: octagon-alert
- copy JSON: copy

Buttons with familiar operations should use icon + tooltip. Text-only buttons are
allowed for primary workflow commands such as `Create Plan`, `Apply`, `Cancel`.

## Motion

Timing:

- hover: `90ms ease-out`
- press: `70ms ease-out`
- panel enter: `140ms ease-out`
- sheet/modal enter: `180ms cubic-out`
- progress update: `160ms linear`
- error shake: `120ms`, max `4px`, only once
- scanning pulse: `1200ms` loop, opacity `0.45 -> 1.0 -> 0.45`

Rules:

- Motion must be subtle and never block device operations.
- Progress bars use linear motion; do not spring.
- Dangerous confirmation sheets fade in and slide up `8px`.
- Config diffs highlight changed rows for `900ms`, then settle to a stable dirty
  marker.

## Components

### Sidebar

- width: `216px`
- item height: `36px`
- icon box: `20px`
- item radius: `6px`
- horizontal padding: `10px`
- selected background: `#EAF2FF`
- selected text: `#1F4EAD`
- item spacing: `4px`

Items:

```text
Dashboard
Flash
Memory
Config
Project
Catalog
Automation
```

Each item can show a small state dot:

- green: ready
- amber: attention
- red: blocked
- purple: AI available

### Top Bar

Height: `56px`.

Left area:

- current device name or `No device`
- family and chip id
- transport chip: USB / Serial / Mock

Right area:

- scan button
- JSON copy button
- settings button

Device state chip:

- height: `24px`
- radius: `12px`
- padding: `8px horizontal`
- caption text
- icon optional

### Status Bar

Height: `36px`.

Content:

- latest event summary
- operation state
- safety state
- copy event JSON button

Use monospace only for event ids and error codes.

### Panels

Panel:

- radius: `8px`
- border: `1px solid border`
- background: `surface`
- padding: `14px`
- title row height: `28px`

Repeated item cards:

- radius: `6px`
- border: `1px solid border`
- padding: `10px`
- min height: `48px`

No nested cards. Use table rows, split panes, or bordered groups instead.

### Buttons

Heights:

- primary: `36px`
- secondary: `32px`
- icon-only: `32 x 32px`
- compact table action: `28px`

Primary read action:

- background `blue`
- text white

Primary apply action:

- background `green`
- text white

Destructive final apply:

- background `red`
- text white
- requires confirmation sheet

Plan action:

- background `amber`
- text `#1F2328`

Disabled:

- background `#E6E8EB`
- text `#8C959F`
- no hover color shift
- tooltip or detail reason required

### Tables

Header:

- height `32px`
- caption text
- background `surface-muted`

Rows:

- height `36px`
- border bottom `1px #EAEEF2`
- selected row bg `#EAF2FF`
- dirty row bg `#FFF8E1`
- danger row bg `#FFEBE9`

Columns should use stable widths for:

- address: `92px`
- raw hex: `108px`
- bit range: `72px`
- status chip: `88px`

### Modal / Sheet

Use modal sheets only for:

- destructive confirmation
- config diff apply
- project apply
- unrecoverable error details

Size:

- width: `560px`
- max height: `80% window`
- padding: `20px`
- radius: `10px`

Danger confirmation must show:

- operation
- detected chip
- target region/config
- exact risk
- verification step
- typed confirmation only for high-risk config fields

## Page Specifications

### Dashboard

Layout:

```text
2 columns: left 58%, right 42%
top row: device summary full width
left: available operations and next recommended step
right: permissions, recent events, AI status
```

Device summary panel:

- height: `128px`
- left: chip icon, device name, family, chip id
- center: flash/eeprom sizes
- right: transport and state

No device state:

- title: `No WCH ISP device`
- primary action: `Scan`
- secondary hint: USB and serial detection notes
- all write actions disabled

### Flash

Layout:

- left pane `360px`: firmware source and metadata
- right pane: operation plan
- bottom: progress and report

Firmware source:

- local file picker
- recent files
- optional URL/release source later
- parsed format badge: BIN / HEX / ELF / Unknown

Plan preview steps:

1. connect
2. identify
3. validate chip
4. erase
5. program
6. verify
7. reset

Only after a valid plan exists does `Apply Flash` appear.

### Memory

Layout:

- region list left `280px`
- selected region inspector right

Region card content:

- name
- address range
- size
- supported operation chips
- danger level

Inspector:

- read/export row
- write/import row
- erase row
- hex preview placeholder

Unsupported operations are visible but disabled with reason.

### Config

Layout:

```text
left: register list, 280px
center: field table
right: selected field inspector, 300px
bottom: diff drawer when dirty
```

Register list row:

- register name
- offset
- dirty marker
- danger marker if any field is high risk

Field table:

- field
- bits
- current
- target
- reset
- risk

Inspector:

- description
- enum meaning
- raw bit display
- control
- reset requirement
- constraints

Diff drawer:

- height: `180px`
- appears only when target differs from current
- includes `Plan Config Apply`
- final write hidden until plan is valid

### Project

Layout:

- top: project file path and validation state
- center: project intent summary
- right: detected device comparison
- bottom: generated operation plan

States:

- no project
- parsed project
- target mismatch
- ready to plan
- ready to apply

### Catalog

Layout:

- search at top
- family list left
- variants table center
- selected variant detail right

Filters:

- USB supported
- serial supported
- has config registers
- has data flash
- family prefix

This page is read-only.

### Automation

Purpose: make the app transparent to AI and advanced users.

Content:

- current state JSON
- last operation JSON
- available AI commands
- MCP status placeholder
- copy buttons

Use purple as the accent color, but keep panels white and technical.

## Safety Visual Language

Risk levels:

- `none`: neutral
- `low`: blue/info
- `medium`: amber
- `high`: red
- `unknown`: gray with question mark

High-risk examples:

- read protection
- debug lock
- boot mode
- reset pin disable
- flash write protection
- raw config register edit

High-risk actions require:

- red icon
- explicit risk text
- plan preview
- typed confirmation if irreversible or recovery is uncertain

## Responsive Rules

Minimum `880 x 620`:

- sidebar remains visible
- right inspector can collapse into bottom drawer
- tables keep address/raw/status columns fixed
- text elides instead of resizing controls

Wide `1280+`:

- Config page uses 3-pane layout
- Dashboard recent events can expand to 8 rows

No mobile layout is required.

## Computer Use QA Checklist

Every UI implementation round must capture screenshots for:

- `dashboard-no-device`
- `catalog-search-ch59`
- `flash-no-firmware`
- `flash-plan-ready` with mock device
- `memory-region-list` with mock CH592
- `config-readonly-ch552`
- `config-diff-high-risk`
- `project-target-mismatch`
- `automation-json-state`

For each screenshot, record:

- viewport size
- app state
- enabled destructive controls
- disabled reasons visible
- text overlap: yes/no
- clipped text: yes/no
- visual hierarchy verdict
- safety verdict

Failure criteria:

- destructive action visible as primary without a valid plan
- disabled action has no reason
- long chip/register names overlap controls
- config reserved bits appear editable by default
- status colors conflict with risk meaning
- page requires scrolling to see the primary safety warning

## Implementation Order

1. Replace current small card UI with app shell.
2. Implement Dashboard no-device and mock-device states.
3. Implement Catalog page using `meowisp ai catalog --json` data shape.
4. Implement Flash plan preview without execution changes.
5. Implement Config read-only layout with mock data.
6. Add Computer Use screenshot fixtures.
7. Only then add config editing and guarded writes.

## Current UI Gap

The current UI is useful as an early flashing prototype, but it is not the target
design:

- window is too small for a universal engineering tool
- navigation is missing
- Dashboard/Catalog/Automation surfaces are missing
- actions are presented before operation plans exist
- config registers have no first-class space
- visual language is friendly but too board-specific

The redesign should treat the existing UI as a prototype, not as a style guide.
