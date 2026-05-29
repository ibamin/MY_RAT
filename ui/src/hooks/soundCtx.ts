import { createContext } from 'react';
import type { useGameSound } from './useGameSound';

export type GameSoundContextType = ReturnType<typeof useGameSound>;

export const SoundContext = createContext<GameSoundContextType | null>(null);
