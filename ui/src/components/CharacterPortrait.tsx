import { useMemo } from 'react';
import { motion } from 'framer-motion';
import type { CharacterClass } from '../lib/types';

type PortraitSize = 'sm' | 'md' | 'lg' | 'xl';

const SIZE_MAP: Record<PortraitSize, number> = {
  sm: 40,
  md: 56,
  lg: 80,
  xl: 120,
};

const CLASS_CONFIG: Record<
  CharacterClass,
  {
    primaryColor: string;
    secondaryColor: string;
    glowColor: string;
    emblem: string;
    hairStyle: 'short' | 'long' | 'spiky' | 'bob' | 'tied' | 'buzz';
  }
> = {
  striker: {
    primaryColor: '#00f0ff',
    secondaryColor: '#0088aa',
    glowColor: 'rgba(0,240,255,0.3)',
    emblem: '⚔',
    hairStyle: 'spiky',
  },
  phantom: {
    primaryColor: '#39ff14',
    secondaryColor: '#1a8a0a',
    glowColor: 'rgba(57,255,20,0.3)',
    emblem: '🐧',
    hairStyle: 'long',
  },
  sentinel: {
    primaryColor: '#bf5af2',
    secondaryColor: '#7a3a9e',
    glowColor: 'rgba(191,90,242,0.3)',
    emblem: '🍎',
    hairStyle: 'bob',
  },
  commander: {
    primaryColor: '#ffe156',
    secondaryColor: '#aa9530',
    glowColor: 'rgba(255,225,86,0.3)',
    emblem: '🎖️',
    hairStyle: 'short',
  },
  analyst: {
    primaryColor: '#bf5af2',
    secondaryColor: '#8040b0',
    glowColor: 'rgba(191,90,242,0.3)',
    emblem: '📊',
    hairStyle: 'tied',
  },
  operator: {
    primaryColor: '#00f0ff',
    secondaryColor: '#0077aa',
    glowColor: 'rgba(0,240,255,0.3)',
    emblem: '🎯',
    hairStyle: 'buzz',
  },
};

/**
 * Generate a deterministic seed from a string (hostname/id).
 * Used to vary facial features per-agent while keeping them stable.
 */
function hashSeed(input: string): number {
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    hash = (hash * 31 + input.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

function seededRandom(seed: number, index: number): number {
  const x = Math.sin(seed + index * 127.1) * 43758.5453;
  return x - Math.floor(x);
}

function HairSvg({
  style,
  color,
  s,
}: {
  style: 'short' | 'long' | 'spiky' | 'bob' | 'tied' | 'buzz';
  color: string;
  s: number; // size multiplier
}) {
  const cx = 50;

  switch (style) {
    case 'spiky':
      return (
        <g>
          <path
            d={`M${cx - 18 * s} ${30 * s} Q${cx - 10 * s} ${8 * s} ${cx - 5 * s} ${14 * s} L${cx} ${6 * s} L${cx + 5 * s} ${14 * s} Q${cx + 10 * s} ${8 * s} ${cx + 18 * s} ${30 * s}`}
            fill={color}
            opacity={0.9}
          />
          <path
            d={`M${cx - 12 * s} ${18 * s} L${cx - 8 * s} ${10 * s} L${cx - 3 * s} ${16 * s}`}
            fill={color}
            opacity={0.7}
          />
          <path
            d={`M${cx + 3 * s} ${16 * s} L${cx + 8 * s} ${10 * s} L${cx + 12 * s} ${18 * s}`}
            fill={color}
            opacity={0.7}
          />
        </g>
      );
    case 'long':
      return (
        <g>
          <ellipse cx={cx} cy={28 * s} rx={20 * s} ry={18 * s} fill={color} opacity={0.85} />
          <path
            d={`M${cx - 18 * s} ${30 * s} Q${cx - 22 * s} ${55 * s} ${cx - 16 * s} ${65 * s}`}
            stroke={color}
            strokeWidth={4 * s}
            fill="none"
            opacity={0.6}
          />
          <path
            d={`M${cx + 18 * s} ${30 * s} Q${cx + 22 * s} ${55 * s} ${cx + 16 * s} ${65 * s}`}
            stroke={color}
            strokeWidth={4 * s}
            fill="none"
            opacity={0.6}
          />
        </g>
      );
    case 'bob':
      return (
        <g>
          <ellipse cx={cx} cy={26 * s} rx={19 * s} ry={16 * s} fill={color} opacity={0.85} />
          <path
            d={`M${cx - 18 * s} ${30 * s} Q${cx - 20 * s} ${44 * s} ${cx - 14 * s} ${46 * s}`}
            stroke={color}
            strokeWidth={5 * s}
            fill="none"
            opacity={0.7}
          />
          <path
            d={`M${cx + 18 * s} ${30 * s} Q${cx + 20 * s} ${44 * s} ${cx + 14 * s} ${46 * s}`}
            stroke={color}
            strokeWidth={5 * s}
            fill="none"
            opacity={0.7}
          />
        </g>
      );
    case 'tied':
      return (
        <g>
          <ellipse cx={cx} cy={26 * s} rx={18 * s} ry={15 * s} fill={color} opacity={0.85} />
          <path
            d={`M${cx + 12 * s} ${22 * s} Q${cx + 24 * s} ${18 * s} ${cx + 22 * s} ${36 * s}`}
            stroke={color}
            strokeWidth={3 * s}
            fill="none"
            opacity={0.7}
          />
        </g>
      );
    case 'buzz':
      return (
        <ellipse cx={cx} cy={26 * s} rx={16 * s} ry={14 * s} fill={color} opacity={0.6} />
      );
    case 'short':
    default:
      return (
        <g>
          <ellipse cx={cx} cy={25 * s} rx={17 * s} ry={14 * s} fill={color} opacity={0.8} />
          <path
            d={`M${cx - 16 * s} ${28 * s} Q${cx - 14 * s} ${18 * s} ${cx} ${16 * s} Q${cx + 14 * s} ${18 * s} ${cx + 16 * s} ${28 * s}`}
            fill={color}
            opacity={0.6}
          />
        </g>
      );
  }
}

export function CharacterPortrait({
  characterClass,
  seed = 'default',
  size = 'md',
  status,
  animated = true,
  onClick,
}: {
  characterClass: CharacterClass;
  seed?: string;
  size?: PortraitSize;
  status?: 'online' | 'offline' | 'unknown';
  animated?: boolean;
  onClick?: () => void;
}) {
  const config = CLASS_CONFIG[characterClass];
  const px = SIZE_MAP[size];
  const s = px / 100;

  const features = useMemo(() => {
    const h = hashSeed(seed);
    return {
      eyeSpacing: 8 + seededRandom(h, 0) * 4,
      eyeSize: 2.2 + seededRandom(h, 1) * 1.2,
      skinTone: Math.floor(seededRandom(h, 2) * 5),
      hairColor: Math.floor(seededRandom(h, 3) * 4),
    };
  }, [seed]);

  const SKIN_TONES = ['#fddbb4', '#f1c27d', '#e0ac69', '#c68642', '#8d5524'];
  const HAIR_COLORS = ['#2c1810', '#4a3226', '#8a6642', '#c4a265'];

  const skinColor = SKIN_TONES[features.skinTone];
  const hairColor = HAIR_COLORS[features.hairColor];

  const statusColor =
    status === 'online'
      ? 'var(--status-victory)'
      : status === 'offline'
        ? 'var(--muted-2)'
        : undefined;

  const portrait = (
    <svg
      width={px}
      height={px}
      viewBox={`0 0 ${100 * s} ${100 * s}`}
      style={{ display: 'block' }}
    >
      <defs>
        <radialGradient id={`bg-${seed}-${characterClass}`} cx="50%" cy="40%" r="60%">
          <stop offset="0%" stopColor={config.primaryColor} stopOpacity={0.15} />
          <stop offset="100%" stopColor="transparent" />
        </radialGradient>
        <filter id={`glow-${seed}-${characterClass}`}>
          <feGaussianBlur in="SourceGraphic" stdDeviation={2 * s} />
        </filter>
      </defs>

      {/* Background circle */}
      <circle
        cx={50 * s}
        cy={50 * s}
        r={48 * s}
        fill="rgba(10,14,26,0.9)"
        stroke={config.primaryColor}
        strokeWidth={1.5 * s}
        strokeOpacity={0.5}
      />
      <circle
        cx={50 * s}
        cy={50 * s}
        r={48 * s}
        fill={`url(#bg-${seed}-${characterClass})`}
      />

      {/* Neck */}
      <rect
        x={44 * s}
        y={52 * s}
        width={12 * s}
        height={14 * s}
        rx={4 * s}
        fill={skinColor}
        opacity={0.9}
      />

      {/* Body/shoulders */}
      <path
        d={`M${26 * s} ${82 * s} Q${30 * s} ${62 * s} ${50 * s} ${60 * s} Q${70 * s} ${62 * s} ${74 * s} ${82 * s}`}
        fill={config.secondaryColor}
        opacity={0.8}
      />
      {/* Collar accent */}
      <path
        d={`M${40 * s} ${62 * s} L${50 * s} ${68 * s} L${60 * s} ${62 * s}`}
        stroke={config.primaryColor}
        strokeWidth={1.5 * s}
        fill="none"
        opacity={0.6}
      />

      {/* Head */}
      <ellipse
        cx={50 * s}
        cy={34 * s}
        rx={16 * s}
        ry={18 * s}
        fill={skinColor}
      />

      {/* Hair */}
      <HairSvg style={config.hairStyle} color={hairColor} s={s} />

      {/* Eyes */}
      <ellipse
        cx={(50 - features.eyeSpacing) * s}
        cy={36 * s}
        rx={features.eyeSize * s}
        ry={(features.eyeSize * 0.8) * s}
        fill="white"
      />
      <circle
        cx={(50 - features.eyeSpacing) * s}
        cy={36 * s}
        r={(features.eyeSize * 0.5) * s}
        fill={config.primaryColor}
      />
      <ellipse
        cx={(50 + features.eyeSpacing) * s}
        cy={36 * s}
        rx={features.eyeSize * s}
        ry={(features.eyeSize * 0.8) * s}
        fill="white"
      />
      <circle
        cx={(50 + features.eyeSpacing) * s}
        cy={36 * s}
        r={(features.eyeSize * 0.5) * s}
        fill={config.primaryColor}
      />

      {/* Eye glow */}
      <circle
        cx={(50 - features.eyeSpacing) * s}
        cy={36 * s}
        r={(features.eyeSize * 0.5) * s}
        fill={config.primaryColor}
        filter={`url(#glow-${seed}-${characterClass})`}
        opacity={0.4}
      />
      <circle
        cx={(50 + features.eyeSpacing) * s}
        cy={36 * s}
        r={(features.eyeSize * 0.5) * s}
        fill={config.primaryColor}
        filter={`url(#glow-${seed}-${characterClass})`}
        opacity={0.4}
      />

      {/* Mouth */}
      <path
        d={`M${45 * s} ${44 * s} Q${50 * s} ${46 * s} ${55 * s} ${44 * s}`}
        stroke={skinColor}
        strokeWidth={1 * s}
        fill="none"
        opacity={0.5}
        filter="brightness(0.8)"
      />

      {/* Status indicator */}
      {statusColor ? (
        <circle
          cx={80 * s}
          cy={80 * s}
          r={6 * s}
          fill={statusColor}
          stroke="rgba(10,14,26,0.9)"
          strokeWidth={2 * s}
        />
      ) : null}
    </svg>
  );

  if (animated) {
    return (
      <motion.div
        className="characterPortrait"
        whileHover={{ scale: 1.08 }}
        transition={{ type: 'spring', stiffness: 400, damping: 20 }}
        onClick={onClick}
        style={{
          cursor: onClick ? 'pointer' : undefined,
          width: px,
          height: px,
          borderRadius: '50%',
          boxShadow: `0 0 ${12 * s}px ${config.glowColor}`,
          position: 'relative',
          flexShrink: 0,
        }}
      >
        {portrait}
      </motion.div>
    );
  }

  return (
    <div
      className="characterPortrait"
      onClick={onClick}
      style={{
        cursor: onClick ? 'pointer' : undefined,
        width: px,
        height: px,
        borderRadius: '50%',
        boxShadow: `0 0 ${12 * s}px ${config.glowColor}`,
        position: 'relative',
        flexShrink: 0,
      }}
    >
      {portrait}
    </div>
  );
}
