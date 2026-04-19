import { SoundContext } from './soundCtx';
import { useGameSound } from './useGameSound';

export function SoundProvider({ children }: { children: React.ReactNode }) {
  const sound = useGameSound();
  return <SoundContext.Provider value={sound}>{children}</SoundContext.Provider>;
}
