/**
 * src/chat/ChatPanel.tsx
 *
 * Blueprint §4: apps/desktop/src/chat/ — streaming chat surface.
 *
 * Phase 1: Chat output is shown in the palette output-row (App.tsx handles
 * `llm:token` events and appends to the output field).
 *
 * Phase 2: This component becomes a full chat surface with:
 * - Message history with user/assistant bubbles
 * - Inline tool execution status (plan steps)
 * - Streaming token animation
 * - Confirmation cards for yellow/red plan steps inline in the chat
 *
 * Today: This file documents the future interface.
 */

interface ChatMessage {
  role: "user" | "assistant" | "tool";
  content: string;
  timestampMs: number;
}

interface ChatPanelProps {
  messages: ChatMessage[];
  streamingToken: string;
  isStreaming: boolean;
}

/**
 * Phase 2 chat panel — renders conversation history with streaming output.
 * Phase 1: Not rendered by App.tsx (palette output-row used instead).
 */
export function ChatPanel({ messages, streamingToken, isStreaming }: ChatPanelProps) {
  return (
    <div className="panel chat-panel">
      <div className="chat-messages">
        {messages.map((msg, i) => (
          <div key={i} className={`chat-bubble chat-bubble--${msg.role}`}>
            <span className="chat-role">{msg.role}</span>
            <span className="chat-content">{msg.content}</span>
          </div>
        ))}
        {isStreaming && (
          <div className="chat-bubble chat-bubble--assistant chat-bubble--streaming">
            <span className="chat-role">assistant</span>
            <span className="chat-content">{streamingToken}<span className="cursor">▋</span></span>
          </div>
        )}
      </div>
    </div>
  );
}
