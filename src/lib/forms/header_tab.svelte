<script lang="ts">
  import {
    defaultHeader,
    parseHeader,
    renderHeader,
    type Header,
  } from "../../backend/header";

  let {
    message,
    onchange,
  }: {
    message?: string;
    onchange?: (message: string) => void;
  } = $props();

  let header: Header = $state(defaultHeader());

  $effect(() => {
    if (message) {
      parseHeader(message).then((parsedHeader) => {
        if (parsedHeader) {
          header = parsedHeader;
        }
      });
    }
  });

  const oninput = () => {
    if (onchange && message) {
      renderHeader(message, header).then((renderedHeader) => {
        if (renderedHeader) {
          onchange(renderedHeader);
        }
      });
    }
  };

  const onfocus = (event: Event) => {
    console.debug("onfocus", event);
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    console.debug("popover", popover);
    if (popover) {
      console.debug("showing popover");
      (popover as HTMLElement).classList.add("show");
    }
  };

  const onblur = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.remove("show");
    }
  };

  const trigger_events_list = $derived(
    header.message_type?.toLowerCase() === "adt"
      ? "trigger_events_adt"
      : header.message_type?.toLowerCase() === "orm"
        ? "trigger_events_orm"
        : "",
  );
</script>

<div class="msh">
  <form>
    <div class="form-group">
      <label for="sending_app">Sending Application</label>
      <input
        type="text"
        id="sending_app"
        bind:value={header.sending_application}
        {oninput}
        placeholder="SAPP"
        maxlength="20"
      />
    </div>
    <div class="form-group">
      <label for="receiving_app">Receiving Application</label>
      <input
        type="text"
        id="receiving_app"
        bind:value={header.receiving_application}
        {oninput}
        placeholder="RAPP"
        maxlength="20"
      />
    </div>
    <div class="form-group">
      <label for="sending_facility">Sending Facility</label>
      <input
        type="text"
        id="sending_facility"
        bind:value={header.sending_facility}
        {oninput}
        placeholder="SFAC"
        maxlength="20"
      />
    </div>
    <div class="form-group">
      <label for="receiving_facility">Receiving Facility</label>
      <input
        type="text"
        id="receiving_facility"
        bind:value={header.receiving_facility}
        {oninput}
        placeholder="RFAC"
        maxlength="20"
      />
    </div>

    <div class="form-group">
      <label for="timestamp">Timestamp</label>
      <input
        type="text"
        id="timestamp"
        bind:value={header.date_time_of_message}
        {oninput}
        {onfocus}
        {onblur}
        placeholder="YYYYMMDDHHMMSS"
        pattern={"(\\{auto\\})|((\\d{4})(\\d{2})(\\d{2})(\\d{2})(\\d{2})(\\d{2})(\\.\\d{1,3})?([+-]\\d{4})?)"}
        maxlength="23"
      />
      <p class="popover">
        <span class="note">Note: </span>Recorded if present, but not used for
        processing. If this field has a timezone offset, that timezone offset is
        used for all date fields in the message that does not have its own
        timezone offset. If there is no timezone offset, then the default
        timezone is used.
      </p>
    </div>
    <div class="form-group">
      <label for="control_id">Control ID</label>
      <input
        type="text"
        id="control_id"
        bind:value={header.control_id}
        {oninput}
        placeholder="1234567890"
        maxlength="20"
      />
    </div>
    <div class="form-group short">
      <label for="message_type">Message Type</label>
      <input
        type="text"
        id="message_type"
        bind:value={header.message_type}
        {oninput}
        placeholder="ADT"
        list="message_types"
        maxlength="3"
      />
    </div>
    <div class="form-group short">
      <label for="trigger_event">Trigger Event</label>
      <input
        type="text"
        id="trigger_event"
        bind:value={header.trigger_event}
        {oninput}
        placeholder="A08"
        list={trigger_events_list}
        maxlength="3"
      />
    </div>

    <div class="form-group short">
      <label for="processing_id">Processing ID</label>
      <input
        type="text"
        id="processing_id"
        bind:value={header.processing_id}
        {oninput}
        placeholder="P"
        list="processing_ids"
        maxlength="1"
      />
    </div>
    <div class="form-group short">
      <label for="version_id">Version ID</label>
      <input
        type="text"
        id="version_id"
        bind:value={header.version_id}
        {oninput}
        placeholder="2.5.1"
        pattern={"(\\d{1,2})(\\.\\d{1,2})?([\\.](\\d{1,2}))?"}
        list="version_ids"
        maxlength="5"
      />
    </div>
    <div class="form-group short">
      <label for="accept_ack">Accept ACK</label>
      <input
        type="text"
        id="accept_ack"
        bind:value={header.accept_acknowledgment_type}
        {oninput}
        placeholder="AL"
        pattern={"(AL|NE|ER|SU)?"}
        list="ack_types"
        maxlength="2"
      />
    </div>
    <div class="form-group short">
      <label for="app_ack">Application ACK</label>
      <input
        type="text"
        id="app_ack"
        bind:value={header.application_acknowledgment_type}
        {oninput}
        placeholder="NE"
        pattern={"(AL|NE|ER|SU)?"}
        list="ack_types"
        maxlength="2"
      />
    </div>
    <div class="form-group short">
      <label for="character_set">Character Set</label>
      <input
        type="text"
        id="character_set"
        bind:value={header.character_set}
        {oninput}
        placeholder="ASCII"
        pattern={"(ASCII)?"}
        list="character_sets"
        maxlength="16"
      />
    </div>
    <datalist id="message_types">
      <option value="ADT" label="Patient Administration"></option>
      <option value="ORM" label="Order Entry"></option>
      <option value="ORU" label="Observation Reporting"></option>
      <option value="ACK" label="Control"></option>
      <option value="MFN" label="Master Files"></option>
      <option value="BTS" label="Order Entry"></option>
    </datalist>
    <datalist id="trigger_events_adt">
      <option value="A01" label="Admit/Visit Notification"></option>
      <option value="A02" label="Transfer a Patient"></option>
      <option value="A03" label="Discharge/End Visit"></option>
      <option value="A04" label="Register a Patient"></option>
      <option value="A05" label="Pre-Admit a Patient"></option>
      <option value="A06" label="Change an Outpatient to an Inpatient"></option>
      <option value="A07" label="Change an Inpatient to an Outpatient"></option>
      <option value="A08" label="Update Patient Information"></option>
      <option value="A12" label="Cancel Transfer"></option>
      <option value="A13" label="Cancel Discharge / End Visit"></option>
      <option value="A45" label="Move Visit Information - Visit Number"
      ></option>
      <option value="A49" label="Change Patient Account Number"></option>
      <option value="A50" label="Change Visit Number"></option>
    </datalist>
    <datalist id="trigger_events_orm">
      <option value="O01" label="General Order"></option>
    </datalist>
    <datalist id="processing_ids">
      <option value="P" label="Production"></option>
      <option value="D" label="Debugging"></option>
      <option value="T" label="Training"></option>
    </datalist>
    <datalist id="version_ids">
      <option value="2.3"></option>
      <option value="2.3.1"></option>
      <option value="2.4"></option>
      <option value="2.5.1"></option>
    </datalist>
    <datalist id="ack_types">
      <option value="AL" label="Always"></option>
      <option value="NE" label="Never"></option>
      <option value="ER" label="Error"></option>
      <option value="SU" label="Success"></option>
    </datalist>
    <datalist id="character_sets">
      <option value="ASCII" label="The printable 7-bit ASCII character set."
      ></option>
    </datalist>
  </form>
</div>

<style></style>
