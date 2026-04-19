import { motion } from 'framer-motion';

export function CombatResult(props: {
  verdict: string;
  stepName: string;
  summary?: string;
  onDismiss?: () => void;
}) {
  const { verdict, stepName, summary, onDismiss } = props;
  const upper = verdict.toUpperCase();
  const isPassed = upper === 'PASS';
  const isFailed = upper === 'FAIL';

  const className = isPassed
    ? 'battleResultPass'
    : isFailed
      ? 'battleResultFail'
      : 'battleResultPending';
  const label = isPassed
    ? 'OPERATION SUCCESS'
    : isFailed
      ? 'OPERATION FAILED'
      : 'OPERATION PENDING';
  const glowColor = isPassed
    ? 'var(--status-victory)'
    : isFailed
      ? 'var(--status-defeat)'
      : 'var(--status-pending)';

  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: 'easeOut' }}
    >
      <div
        style={{
          fontSize: 13,
          letterSpacing: '0.1em',
          textTransform: 'uppercase',
          color: 'var(--muted)',
          marginBottom: 4,
        }}
      >
        {stepName}
      </div>
      <div
        style={{
          fontSize: 20,
          fontWeight: 700,
          color: glowColor,
          textShadow: `0 0 20px ${glowColor}`,
        }}
      >
        {label}
      </div>
      {summary ? (
        <div style={{ marginTop: 8, color: 'var(--muted)', fontSize: 13 }}>{summary}</div>
      ) : null}
      {onDismiss ? (
        <button className="btn" style={{ marginTop: 12 }} onClick={onDismiss}>
          Dismiss
        </button>
      ) : null}
    </motion.div>
  );
}
