# Hermes Help Guide

## Introduction

Hermes is a desktop application designed for composing, editing, and working
with HL7 messages. It's primarily used for development, testing, and quality
assurance of healthcare integration systems that communicate using the HL7 v2.x
messaging standard.

Whether you're building interfaces for Electronic Health Records (EHR),
Laboratory Information Systems (LIS), or other healthcare applications, Hermes
provides an intuitive interface for creating and manipulating HL7 messages.

---

## Understanding HL7

### What is HL7?

HL7 (Health Level 7) is an international standard for exchanging healthcare
information between computer systems. HL7 v2.x messages are text-based and use a
specific format with delimiters to organise data.

### Message Structure

An HL7 message consists of **segments**, which are like rows in a table. Each
segment contains **fields**, which hold specific pieces of information.

**Example HL7 Message:**
```
MSH|^~\&|SENDING_APP|SENDING_FAC|RECEIVING_APP|RECEIVING_FAC|20250114120000||ADT^A01|MSG001|P|2.5
PID|1||12345678^^^MRN||DOE^JOHN^A||19800115|M|||123 MAIN ST^^ANYTOWN^CA^12345
PV1|1|I|ICU^101^1|||||||||||||||V12345
```

### Segments

Each segment starts with a three-letter code that identifies its purpose:

- **MSH** (Message Header): Required first segment containing routing and
  message type information
- **PID** (Patient Identification): Patient demographic information
- **PV1** (Patient Visit): Visit/encounter information
- **OBX** (Observation/Result): Lab results, vital signs, or other observations
- **ORC** (Common Order): Order control information
- **And many more...**

### Fields, Components, and Repetitions

Within each segment:

- **Fields** are separated by the pipe character `|`
- **Components** within a field are separated by `^`
- **Repetitions** of a field are separated by `~`
- **Subcomponents** are separated by `&`

**Example:**
```
PID|1||12345678^^^MRN||DOE^JOHN^A||19800115|M
```

Breaking this down:
- `PID` = Segment ID
- `1` = Field 1 (Set ID)
- `` = Field 2 (empty)
- `12345678^^^MRN` = Field 3 (Patient Identifier List)
  - `12345678` = Component 1 (ID Number)
  - `` = Component 2 (empty)
  - `` = Component 3 (empty)
  - `MRN` = Component 4 (Identifier Type)
- `DOE^JOHN^A` = Field 5 (Patient Name)
  - `DOE` = Component 1 (Last Name)
  - `JOHN` = Component 2 (First Name)
  - `A` = Component 3 (Middle Initial)

### Encoding Characters

The MSH segment defines the delimiters used throughout the message:

```
MSH|^~\&|...
```

- `|` = Field separator
- `^` = Component separator
- `~` = Repetition separator
- `\` = Escape character
- `&` = Subcomponent separator

---

## Getting Started

### Basic Workflow

1. **Create a new message** by clicking the "New" button in the toolbar
2. **Fill in the MSH (Message Header)** using the segment tab interface
3. **Add additional segments** using the "+" button
4. **Populate fields** either through the tab forms or by editing the raw
   message
5. **Save your message** to a file for later use or testing

### Two Ways to Edit

Hermes gives you two ways to work with messages:

1. **Segment Tabs** (Top Panel): Form-based editing with labelled fields and
   validation
2. **Message Editor** (Middle Panel): Direct text editing of the raw HL7 message

Both views are synchronised - changes in one are immediately reflected in the
other.

---

## Interface Overview

Hermes uses a three-panel layout:

### Top Panel: Segment Tabs

Shows one tab for each segment in your message. Click a tab to view and edit
that segment's fields in a form layout.

### Middle Panel: Message Editor

Displays the complete HL7 message as raw text with syntax highlighting. You can
type directly here to edit the message.

**Resizing the Editor:**
- Drag the handle at the bottom of the message editor to resize it
- The editor can be between 100 pixels and 60% of your screen height
- Your preferred size is saved automatically
- The handle turns red when you've reached the minimum or maximum size

### Bottom Panel: Cursor Description

Shows information about the field where your cursor is currently positioned in
the message editor, including:
- Field path (e.g., "MSH.3.1")
- Field name (e.g., "Sending Application")
- Group hierarchy (e.g., "Message Header → Sending Application")
- Field type/specification

This helps you understand what each field is for as you navigate the message.

---

## Message Editor

### Syntax Highlighting

The message editor colour-codes different parts of your HL7 message to make it
easier to read:

- **Segment identifiers** (MSH, PID, etc.): Bright blue
- **MSH segment header**: Green
- **Field separators** (|): Grey
- **Timestamps and dates**: Purple
- **Template/variable data**: Gold
- **Errors**: Red

### Keyboard Shortcuts

- **Tab**: Jump to the next field in the message
- **Shift+Tab**: Jump to the previous field in the message
- **Ctrl+Enter** (Windows/Linux) or **Cmd+Enter** (Mac): Open Communication Drawer (Send tab)

### Copy to Clipboard

Hover over the top-right corner of the message editor to reveal a copy button.
Click it to copy the entire message to your clipboard. The button changes to a
checkmark to confirm the copy.

---

## Segment Tabs

### Viewing and Editing Segments

Each segment in your message has its own tab at the top of the interface. The
tabs are labelled with the segment name (e.g., "MSH", "PID", "PV1").

**For repeated segments**, tabs are numbered:
- OBX (1)
- OBX (2)
- OBX (3)

Click a tab to view and edit that segment's fields.

### Adding New Segments

Click the **"+"** button at the end of the tab row to add a new segment. A
dropdown menu will appear with available segment types. Select the segment you
want to add, and it will be appended to your message with a new tab created.

### Field Editing

Within each tab, you'll see a form with labelled fields:

- **Field labels** show the field name and ID (e.g., "Sending Application
  (MSH.3)")
- **Required fields** are marked and enforced
- **Input validation** checks minimum/maximum length and patterns
- **Auto-complete suggestions** appear for fields with predefined values
- **Field notes** appear as popovers when you focus on a field (if documentation
  is available)

### Tabs Follow Cursor

By default, when you click in the message editor, the segment tabs automatically
switch to show the segment where your cursor is located. You can disable this
    behaviour in **Settings** if you prefer to manually switch tabs.

---

## Cursor Description Panel

The bottom panel shows detailed information about the field at your current
cursor position:

- **Field Path**: The exact location in the HL7 structure (e.g., "PID.5.1")
- **Field Name**: Human-readable name (e.g., "Patient Last Name")
- **Group Hierarchy**: The organisational path (e.g., "Patient Identification →
  Patient Name → Last Name")
- **Specification**: Field type and additional metadata

This panel updates in real-time as you move your cursor through the message
editor, helping you understand the structure and purpose of each field.

---

## Wizards

Wizards help you quickly populate segments with data from your database.
Currently, three wizards are available:

- **Header Wizard** (MSH segment)
- **Patient Wizard** (PID segment)
- **Visit Wizard** (PV1 segment)

### Setting Up Database Connection

Before using wizards, configure your database connection:

1. Click the **Settings** button in the toolbar
2. Under "Database Connection", enter:
   - **Host**: Your database server address
   - **Port**: Database port (default: 1433)
   - **Database**: Database name
   - **Username**: Your database username
   - **Password**: Your database password
3. Close the settings - your configuration is saved automatically

### Using the Header Wizard (MSH)

The Header Wizard helps you populate message routing information from interface
configurations stored in your database.

1. Navigate to the **MSH tab**
2. Click the **magic wand icon** (✨) on the tab
3. The wizard will auto-detect the message type and trigger event from your
current message
4. Or, manually select:
   - **Message Type**: ADT or ORM
   - **Trigger Event**: Context-specific options (e.g., A01, A04, A08 for ADT)
5. Click **Search**
6. Select an interface from the results table showing:
   - Interface name
   - Sending Application
   - Sending Facility
   - Receiving Application
   - Receiving Facility
7. Toggle **Override Segment** if you want to replace all MSH data (otherwise,
fields are merged)
8. Click **Apply** to update your message

### Using the Patient Wizard (PID)

The Patient Wizard searches for patients and populates patient demographic
information.

1. Navigate to the **PID tab**
2. Click the **magic wand icon** (✨)
3. Enter at least one search criterion:
   - **Patient Name**
   - **Patient ID**
   - **Patient MRN**
4. Click **Search**
5. Select a patient from the results table showing:
   - Patient ID
   - MRN
   - Last Name
   - First Name
   - Date of Birth
   - Sex
6. Toggle **Override Segment** if you want to replace all PID data
7. Click **Apply** to update your message

### Using the Visit Wizard (PV1)

The Visit Wizard finds visit/encounter information based on the patient in your
current message.

1. Navigate to the **PV1 tab**
2. Click the **magic wand icon** (✨)
3. The wizard automatically searches for visits matching the patient in your PID
segment
4. Select a visit from the results table showing:
   - Sequence number
   - Location
   - Type (Inpatient/Outpatient/Emergency)
   - Account Number
   - Admission Date
   - Discharge Date
5. Toggle **Override Segment** if you want to replace all PV1 data
6. Click **Apply** to update your message

### Wizard Tips

- **Override Segment**: When enabled, the wizard replaces all data in the
  segment. When disabled, it only fills in empty fields and merges data
  carefully.
- **Auto-population**: The Header Wizard can detect your message type and
  trigger event from the current message to streamline the search.
- **No Results**: If your search returns no results, try adjusting your search
  criteria or verify your database connection.

---

## File Operations

All file operations are available through both the toolbar buttons and the native
**File** menu with standard keyboard shortcuts (see [Keyboard Shortcuts](#keyboard-shortcuts-reference)).

### Creating a New Message

Click the **New** button in the toolbar (or use **Cmd/Ctrl+N**) to create a fresh
HL7 message. Hermes will generate a basic MSH segment with default field separators:

``` MSH|^~\&| ```

### Opening an Existing Message

1. Click the **Open** button in the toolbar (or use **Cmd/Ctrl+O**)
2. Browse to and select an `.hl7` file
3. The message will load into the editor and segment tabs

### Saving Your Message

**Save** (**Cmd/Ctrl+S**):
- Click the **Save** button (or use **Cmd/Ctrl+S**) to save to the currently open file
- The Save button and menu item are only enabled when you have an open file and unsaved changes

**Save As** (**Cmd/Ctrl+Shift+S**):
- Click the **Save As** button (or use **Cmd/Ctrl+Shift+S**) to save to a new file location
- Choose a filename and location in the file dialog
- Your message will be saved as an `.hl7` text file

### File Path Tracking

When you open or save a file, Hermes remembers the file path. This allows the
Save button to work without prompting you for a location each time.

---

## Communication

Hermes can send and receive HL7 messages over the network using the MLLP
(Minimal Lower Layer Protocol) standard, which is how most healthcare systems
exchange HL7 v2.x messages. The Communication Drawer provides a convenient,
non-blocking interface for both operations.

### Opening the Communication Drawer

Click the **Communication** button in the toolbar to open the drawer. The button
shows the current state:

- **Grey**: Not listening, drawer closed
- **Green**: Listen server is active
- **Purple**: Drawer is open

A red badge appears on the button when there are unread received messages.

### Send Tab

The Send tab lets you send the current message to a remote MLLP server and view
the response.

**Configuration:**

- **Host**: The server address to send to (IP address or hostname)
- **Port**: The MLLP port number (commonly 2575, but varies by system)
- **Timeout**: How long to wait for a response (in seconds)

These settings are saved automatically and persist across sessions.

**Sending a Message:**

1. Configure the host, port, and timeout
2. Click **Send**
3. The status area shows connection progress
4. If successful, the response appears in the Response panel with syntax
   highlighting

**Understanding Responses:**

Most receiving systems respond with an ACK (acknowledgment) message. Common ACK
codes in MSA-1:

- **AA**: Application Accept - message was processed successfully
- **AE**: Application Error - message had errors but was understood
- **AR**: Application Reject - message was rejected

### Listen Tab

The Listen tab runs an MLLP server to receive incoming HL7 messages from other
systems.

**Starting the Server:**

1. Enter the **Port** to listen on (commonly 2575)
2. Click **Start Listening**
3. The status shows "Listening on port X" when active

Click **Stop Listening** to shut down the server.

**Viewing Received Messages:**

Received messages appear in the centre list with:

- **●** (filled circle): Unread message
- **○** (empty circle): Read message
- **Message type**: Extracted from MSH-9 (e.g., "ADT^A01")
- **Time**: When the message was received

Click a message to view it in the right panel with syntax highlighting.

**Loading Messages to Editor:**

Click **Load to Editor** to copy a received message into the main editor. This
is useful for modifying and resending messages during testing.

### Understanding MLLP

MLLP wraps HL7 messages with special characters to mark the start and end:

- **Start Block**: `0x0B` (vertical tab character)
- **End Block**: `0x1C` (file separator) followed by `0x0D` (carriage return)

Hermes handles this framing automatically. You just work with the HL7 message
content.

---

## Settings

Access settings by clicking the **Settings** button in the toolbar.

### Editor Preferences

**Tabs Follow Cursor:**
- When enabled (default), clicking in the message editor automatically switches
  to the tab for that segment
- Disable this if you prefer to manually control which tab is active

### Database Connection

Configure the database connection used by the wizards:

- **Host**: Database server address
- **Port**: Database port number (default: 1433)
- **Database**: Name of the database
- **Username**: Your database login
- **Password**: Your database password

All settings are saved automatically with a short delay after you change them.

---

## Keyboard Shortcuts Reference

### File Operations

| Shortcut | Action |
|----------|--------|
| **Ctrl+N** (Windows/Linux) / **Cmd+N** (Mac) | Create new message |
| **Ctrl+O** (Windows/Linux) / **Cmd+O** (Mac) | Open file |
| **Ctrl+S** (Windows/Linux) / **Cmd+S** (Mac) | Save file (when enabled) |
| **Ctrl+Shift+S** (Windows/Linux) / **Cmd+Shift+S** (Mac) | Save As |

### Message Editor

| Shortcut | Action |
|----------|--------|
| **Tab** | Jump to next field in message editor |
| **Shift+Tab** | Jump to previous field in message editor |
| **Ctrl+Enter** (Windows/Linux) / **Cmd+Enter** (Mac) | Open Communication Drawer (Send tab) |

### Help

| Shortcut | Action |
|----------|--------|
| **F1** | Open Help window |

---

## Tips & Tricks

### Efficient Editing

- **Use both editors**: The segment tabs are great for structured editing with
  validation, while the message editor is faster for quick text changes
- **Tab navigation**: Use Tab and Shift+Tab to quickly move between fields
  without reaching for the mouse
- **Copy and modify**: Start with an existing message that's similar to what you
  need, then modify it

### Working with Wizards

- **Configure once**: Set up your database connection in Settings once, and all
  wizards will use it
- **Override wisely**: Use the "Override Segment" toggle carefully - when
  disabled, wizards preserve your manual edits
- **Patient-first workflow**: When building ADT messages, use the Patient Wizard
  first, then the Visit Wizard automatically searches for that patient's visits

### Understanding Field Structure

- **Watch the cursor panel**: As you navigate the message editor, the cursor
  description panel teaches you the HL7 structure
- **Field popovers**: Click into fields in the segment tabs to see documentation
  popovers explaining what each field is for
- **Syntax highlighting**: Color coding helps you quickly identify segments,
  separators, and data types

### File Organization

- **Naming convention**: Consider naming files with the message type and purpose
  (e.g., `ADT_A01_admission.hl7`, `ORM_O01_lab_order.hl7`)
- **Template library**: Save commonly-used message templates for quick reuse
- **Version control**: Keep your `.hl7` files in version control if you're
  maintaining test suites

### Validation

- **Required fields**: The segment tab editor enforces required fields - use it
  to ensure your message is complete
- **Field length**: Pay attention to min/max length validation to avoid
  truncation issues
- **Auto-complete**: Use the auto-complete suggestions for fields with
  predefined values to ensure valid codes

---

## Getting Help

### Additional Resources

- **HL7 Version 2.x Documentation**: For detailed specifications of segments,
  fields, and data types
- **Your Organization's Interface Specifications**: Check your specific
  implementation guide for required fields and formats

### Troubleshooting

**Wizard shows "No results":**
- Verify your database connection settings
- Check that your search criteria matches data in the database
- Ensure the database is accessible from your network

**Fields not validating correctly:**
- Check that required fields are filled in
- Verify min/max length requirements
- Ensure special characters and format match the field specification

**Tabs not following cursor:**
- Check Settings → "Tabs Follow Cursor" is enabled
- Ensure you're clicking within valid segment boundaries

**Communication errors:**
- **Connection refused**: Check that the remote system is running and listening
  on the specified port
- **Timeout**: The server may be slow to respond. Try increasing the timeout in
  the Send tab.
- **Host not found**: Verify the hostname is correct and DNS is resolving
  properly
- **Port already in use**: When starting the Listen server, another application
  may be using the port. Try a different port.
- **Firewall blocking**: Ensure your firewall allows connections on the MLLP
  port (typically 2575)
