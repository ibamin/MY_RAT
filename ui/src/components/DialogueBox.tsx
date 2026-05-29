const variantColors: Record<string, string> = {
  info: 'var(--neon-cyan)',
  warning: 'var(--neon-yellow)',
  error: 'var(--neon-pink)',
  success: 'var(--neon-green)',
};

export function DialogueBox(props: {
  speaker: string;
  text: string;
  variant?: 'info' | 'warning' | 'error' | 'success';
}) {
  const { speaker, text, variant = 'info' } = props;
  const color = variantColors[variant];
  return (
    <div className="dialogueBox" style={{ borderColor: color }}>
      <div className="dialogueSpeaker" style={{ color }}>{speaker}</div>
      <div className="dialogueText">{text}</div>
    </div>
  );
}
