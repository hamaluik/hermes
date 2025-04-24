<script lang="ts">
  import {
    defaultPatient,
    parsePatient,
    renderPatient,
    type Patient,
  } from "../../backend/patient";

  let {
    message,
    onchange,
  }: {
    message?: string;
    onchange?: (message: string) => void;
  } = $props();

  const phoneNumberPattern = "\\(\\d{3}\\)\\d{3}-\\d{4}(?:\\s[Xx]\\d{(1, 5)})?";

  let patient: Patient = $state(defaultPatient());

  $effect(() => {
    if (message) {
      parsePatient(message).then((parsedPatient) => {
        if (parsedPatient) {
          patient = parsedPatient;
        }
      });
    }
  });

  const oninput = () => {
    if (onchange && message) {
      renderPatient(message, patient).then((rendered) => {
        onchange(rendered);
      });
    }
  };

  const onfocus = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    if (popover) {
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
</script>

<div class="patient">
  <form>
    <fieldset>
      <legend>Name</legend>
      <div class="form-group short">
        <label for="name_prefix">Prefix Code</label>
        <input
          type="text"
          id="name_prefix"
          bind:value={patient.name.prefix}
          {oninput}
          placeholder=""
          maxlength="2"
          list="name_prefix_codes"
        />
      </div>
      <div class="form-group">
        <label for="first_name">First</label>
        <input
          type="text"
          id="first_name"
          bind:value={patient.name.first}
          maxlength="30"
          {oninput}
          placeholder="Mickey"
        />
      </div>
      <div class="form-group short">
        <label for="middle_name">Middle</label>
        <input
          type="text"
          id="middle_name"
          maxlength="30"
          bind:value={patient.name.middle}
          {oninput}
          placeholder="Mickey"
        />
      </div>
      <div class="form-group">
        <label for="last_name">Last</label>
        <input
          type="text"
          id="last_name"
          maxlength="50"
          bind:value={patient.name.last}
          {oninput}
          placeholder="Mouse"
        />
      </div>
      <div class="form-group short">
        <label for="name_suffix">Suffix</label>
        <input
          type="text"
          id="name_suffix"
          bind:value={patient.name.suffix}
          {oninput}
          maxlength="4"
          placeholder=""
        />
      </div>
    </fieldset>
    <fieldset>
      <legend>IDs</legend>
      <div class="form-group">
        <label for="mrn">Medical Record Number</label>
        <input
          type="text"
          id="mrn"
          bind:value={patient.mrn}
          {oninput}
          {onfocus}
          {onblur}
          required
          maxlength="20"
          placeholder="MRN123456"
        />
        <p class="popover">
          <span class="note">Note: </span>The core system uses MRN and Provider
          as a means of recognizing existing patients.
        </p>
      </div>
      <div class="form-group">
        <label for="eid">Enterprise ID</label>
        <input
          type="text"
          id="eid"
          bind:value={patient.eid}
          maxlength="25"
          {oninput}
          {onfocus}
          {onblur}
          placeholder="EID123456"
        />
        <p class="popover">
          <span class="note">Note: </span>The Enterprise ID must be unique for a
          non-merged patient. Null value can affect patient authentication.
        </p>
      </div>
      <div class="form-group">
        <label for="ssn">Social Security Number</label>
        <input
          type="text"
          id="ssn"
          bind:value={patient.ssn}
          {oninput}
          {onfocus}
          {onblur}
          pattern={"\\d{3}-?\\d{2}-?\\d{4}"}
          maxlength="11"
          placeholder="123-45-6789"
        />
        <p class="popover">
          <span class="note">Note: </span>System will accept unformatted
          (123456789) or dash formatted (123-45-6789) values. 9 characters will
          be accepted.
        </p>
      </div>
    </fieldset>
    <fieldset>
      <legend>Demographics</legend>
      <div class="form-group">
        <label for="date_of_birth">Date of Birth</label>
        <input
          type="date"
          id="date_of_birth"
          bind:value={patient.date_of_birth}
          {oninput}
          required
          placeholder="YYYY-MM-DD"
        />
      </div>
      <div class="form-group short">
        <label for="gender_code">Gender Code</label>
        <input
          type="text"
          id="gender_code"
          maxlength="1"
          bind:value={patient.gender_code}
          {oninput}
          {onfocus}
          {onblur}
          required
          placeholder="M"
          list="gender_codes"
        />
        <p class="popover">
          <span class="note">Note: </span>This field is decoded, based on values
          in INT_HL7_CONVERSION. Refer to CD_GENDER table for the core system
          internal values.<br />
          If the sent in sex code is not in CD_GENDER then the message will error
          with AE (if requested).
        </p>
      </div>
      <div class="form-group short">
        <label for="ethnicity_code">Ethnicity Code</label>
        <input
          type="text"
          id="ethnicity_code"
          maxlength="2"
          bind:value={patient.ethnicity_code}
          {oninput}
          {onfocus}
          {onblur}
          placeholder="OT"
          list="ethnicity_codes"
        />
        <p class="popover">
          <span class="note">Note: </span>This field is decoded, based on
          INT_HL7_CONVERSION. Refer to CD_ETHNICITY table for the core system
          internal values.<br />
          If the sent in gender code is not in CD_ETHNICITY then the message will
          error with AE (if requested).
        </p>
      </div>
    </fieldset>
    <fieldset>
      <legend>Address</legend>
      <div class="form-group">
        <label for="address1">Address 1</label>
        <input
          type="text"
          id="address1"
          bind:value={patient.address.address1}
          {oninput}
          placeholder="123 Main St"
        />
      </div>
      <div class="form-group">
        <label for="address2">Address 2</label>
        <input
          type="text"
          id="address2"
          bind:value={patient.address.address2}
          {oninput}
          placeholder=""
        />
      </div>
      <div class="form-group">
        <label for="city">City</label>
        <input
          type="text"
          id="city"
          bind:value={patient.address.city}
          {oninput}
          placeholder="Springfield"
        />
      </div>
      <div class="form-group short">
        <label for="state">State</label>
        <input
          type="text"
          id="state"
          bind:value={patient.address.state}
          {oninput}
          placeholder="IL"
        />
      </div>
      <div class="form-group short">
        <label for="zip">Zip</label>
        <input
          type="text"
          id="zip"
          bind:value={patient.address.zip}
          {oninput}
          placeholder="12345"
        />
      </div>
      <div class="form-group short">
        <label for="country">Country</label>
        <input
          type="text"
          id="country"
          bind:value={patient.address.country}
          {oninput}
          placeholder="USA"
        />
      </div>
      <div class="form-group short">
        <label for="address_type_code">Type Code</label>
        <input
          type="text"
          id="address_type_code"
          bind:value={patient.address.address_type_code}
          {oninput}
          placeholder="H"
          maxlength="1"
          list="address_type_codes"
        />
      </div>
    </fieldset>
    <fieldset>
      <legend>Phone</legend>
      <div class="form-group">
        <label for="home_phone">Home</label>
        <input
          type="text"
          id="home_phone"
          bind:value={patient.home_phone}
          {oninput}
          placeholder="[(999)]999-9999 [X99999]"
          pattern={phoneNumberPattern}
        />
      </div>
      <div class="form-group">
        <label for="business_phone">Business</label>
        <input
          type="text"
          id="business_phone"
          bind:value={patient.business_phone}
          {oninput}
          placeholder="[(999)]999-9999 [X99999]"
          pattern={phoneNumberPattern}
        />
      </div>
    </fieldset>
    <fieldset>
      <legend>Other</legend>
      <div class="form-group">
        <label for="account_number">Account Number</label>
        <input
          type="text"
          id="account_number"
          bind:value={patient.account_number}
          {oninput}
          maxlength="20"
          placeholder="123456789"
        />
      </div>
      <div class="form-group short">
        <label for="status_code">Status Code</label>
        <input
          type="text"
          id="status_code"
          bind:value={patient.status_code}
          {oninput}
          maxlength="1"
          placeholder="A"
          list="status_codes"
        />
      </div>
    </fieldset>
    <datalist id="name_prefix_codes">
      <option value="DR" label="Doctor"></option>
      <option value="FR" label="Father"></option>
      <option value="MI" label="Miss"></option>
      <option value="MR" label="Mr"></option>
      <option value="MS" label="Mrs"></option>
      <option value="MZ" label="Ms"></option>
      <option value="PF" label="Professor"></option>
      <option value="RV" label="Reverend"></option>
      <option value="SR" label="Sister"></option>
    </datalist>
    <datalist id="gender_codes">
      <option value="M" label="Male"></option>
      <option value="F" label="Female"></option>
    </datalist>
    <datalist id="ethnicity_codes">
      <option value="AA" label="African-American"></option>
      <option value="AS" label="Asian"></option>
      <option value="C" label="Caucasian"></option>
      <option value="HS" label="Hispanic"></option>
      <option value="ME" label="Middle Eastern"></option>
      <option value="NA" label="Native American"></option>
      <option value="OT" label="Other"></option>
      <option value="PI" label="Pacific Islander"></option>
      <option value="UK" label="Unknown"></option>
    </datalist>
    <datalist id="address_type_codes">
      <option value="H" label="Home"></option>
      <option value="W" label="Work"></option>
      <option value="B" label="Business"></option>
      <option value="M" label="Mailing"></option>
    </datalist>
    <datalist id="status_codes">
      <option value="A" label="Active"></option>
      <option value="D" label="Deceased"></option>
      <option value="R" label="Deleting"></option>
      <option value="E" label="Emergency"></option>
      <option value="M" label="Merged"></option>
      <option value="P" label="Partial"></option>
    </datalist>
  </form>
</div>

<style>
</style>
