# Network Communication

Hermes uses the MLLP (Minimal Lower Layer Protocol) for HL7 message transport
over TCP.

## Why MLLP

MLLP is the universal standard for HL7 v2.x message transport. Healthcare
systems built over the past decades almost universally support it, making MLLP
essential for interoperability with existing infrastructure. While newer
standards like FHIR use HTTP, legacy systems that Hermes targets expect MLLP.

The protocol adds minimal overhead to TCP. It wraps messages in start and end
bytes without compression or encryption at the protocol level, keeping the
implementation simple and compatible.

## Message Framing

MLLP wraps each message in framing bytes that delimit where messages start and
end within a TCP stream.

A vertical tab byte (0x0B) marks the start of a message. The message content
follows, terminated by a file separator byte (0x1C) and a carriage return
(0x0D). This framing allows multiple messages to flow over a single connection
while remaining distinguishable.

The `hl7-mllp-codec` crate handles framing automatically. The application sends
and receives raw message strings while the codec adds and strips framing bytes.

## Acknowledgment Modes

HL7 defines two acknowledgment modes that determine how receivers confirm
message receipt.

Original mode uses a simple accept/reject acknowledgment. The receiver parses
the message, determines if it's valid, and sends an ACK with either AA (accept)
or AR (reject). This mode suits fire-and-forget workflows where the sender
doesn't need to know if the receiver processed the message successfully.

Enhanced mode adds commit-level acknowledgments. The receiver first acknowledges
receipt with CA (commit accept), then later sends an application acknowledgment
indicating whether processing succeeded. This mode suits workflows requiring
confirmation that the message was not just received but acted upon.

Hermes detects which mode to use by checking MSH.15 and MSH.16 in received
messages. If these fields are present and populated, enhanced mode applies.
Otherwise, original mode handles the acknowledgment.

## Single Listener Constraint

Only one MLLP listener runs at a time. Starting a new listener on a different
port first stops any existing listener. This constraint simplifies state
management and prevents port conflicts.

The constraint reflects typical usage patterns. Users listen on a single port to
receive messages, adjusting the port as needed. Running multiple listeners
simultaneously adds complexity without clear benefit for the intended workflow.

## Timeout Handling

Sending messages includes a configurable timeout that prevents hung connections
from blocking the interface indefinitely. If the remote system doesn't respond
within the timeout, the send operation fails with an error rather than waiting
forever.

The timeout applies to the entire send-receive cycle: connecting, sending the
message, and receiving the acknowledgment. Network problems at any stage trigger
the timeout, giving users feedback when something goes wrong.

## Related Documentation

- [Communication Patterns](communication.md) — Event-driven patterns for
  send/receive
- [Backend Architecture](backend.md) — Long-running task management for the
  listener
