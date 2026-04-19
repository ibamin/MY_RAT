import { useContext } from 'react';
import { SoundContext } from './soundCtx';
import type { GameSoundContextType } from './soundCtx';

export function useSound(): GameSoundContextType {
  const ctx = useContext(SoundContext);
  if (!ctx) throw new Error('useSound must be used within SoundProvider');
  return ctx;
}
