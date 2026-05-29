import { Howl } from 'howler';
import { useCallback, useEffect, useRef, useState } from 'react';

type SoundName =
  | 'click'
  | 'hover'
  | 'success'
  | 'failure'
  | 'alert'
  | 'deploy'
  | 'unlock'
  | 'transition';

type BgmName = 'menu' | 'mission' | 'combat' | 'briefing';

const SFX_SPRITES: Record<SoundName, [number, number]> = {
  click: [0, 150],
  hover: [200, 100],
  success: [400, 600],
  failure: [1100, 500],
  alert: [1700, 400],
  deploy: [2200, 700],
  unlock: [3000, 800],
  transition: [3900, 500],
};

function generateSfx(name: SoundName): void {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);

    const now = ctx.currentTime;

    switch (name) {
      case 'click':
        osc.type = 'square';
        osc.frequency.setValueAtTime(800, now);
        osc.frequency.exponentialRampToValueAtTime(400, now + 0.08);
        gain.gain.setValueAtTime(0.15, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.08);
        osc.start(now);
        osc.stop(now + 0.08);
        break;

      case 'hover':
        osc.type = 'sine';
        osc.frequency.setValueAtTime(1200, now);
        gain.gain.setValueAtTime(0.06, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.05);
        osc.start(now);
        osc.stop(now + 0.05);
        break;

      case 'success':
        osc.type = 'sine';
        osc.frequency.setValueAtTime(523, now);
        osc.frequency.setValueAtTime(659, now + 0.12);
        osc.frequency.setValueAtTime(784, now + 0.24);
        gain.gain.setValueAtTime(0.15, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.4);
        osc.start(now);
        osc.stop(now + 0.4);
        break;

      case 'failure':
        osc.type = 'sawtooth';
        osc.frequency.setValueAtTime(400, now);
        osc.frequency.exponentialRampToValueAtTime(150, now + 0.3);
        gain.gain.setValueAtTime(0.12, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
        osc.start(now);
        osc.stop(now + 0.3);
        break;

      case 'alert':
        osc.type = 'square';
        osc.frequency.setValueAtTime(880, now);
        osc.frequency.setValueAtTime(660, now + 0.1);
        osc.frequency.setValueAtTime(880, now + 0.2);
        gain.gain.setValueAtTime(0.1, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
        osc.start(now);
        osc.stop(now + 0.3);
        break;

      case 'deploy':
        osc.type = 'sine';
        osc.frequency.setValueAtTime(200, now);
        osc.frequency.exponentialRampToValueAtTime(1200, now + 0.4);
        gain.gain.setValueAtTime(0.12, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.5);
        osc.start(now);
        osc.stop(now + 0.5);
        break;

      case 'unlock':
        osc.type = 'sine';
        osc.frequency.setValueAtTime(440, now);
        osc.frequency.setValueAtTime(554, now + 0.1);
        osc.frequency.setValueAtTime(659, now + 0.2);
        osc.frequency.setValueAtTime(880, now + 0.3);
        gain.gain.setValueAtTime(0.15, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.5);
        osc.start(now);
        osc.stop(now + 0.5);
        break;

      case 'transition':
        osc.type = 'sine';
        osc.frequency.setValueAtTime(600, now);
        osc.frequency.exponentialRampToValueAtTime(900, now + 0.15);
        gain.gain.setValueAtTime(0.08, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.2);
        osc.start(now);
        osc.stop(now + 0.2);
        break;
    }
  } catch { /* noop */ }
}

type BgmState = {
  howl: Howl | null;
  name: BgmName | null;
};

export function useGameSound() {
  const [muted, setMuted] = useState(() => {
    try {
      return localStorage.getItem('game_muted') === '1';
    } catch {
      return false;
    }
  });

  const [sfxVolume, setSfxVolume] = useState(() => {
    try {
      const stored = localStorage.getItem('game_sfx_volume');
      return stored ? parseFloat(stored) : 0.5;
    } catch {
      return 0.5;
    }
  });

  const [bgmVolume, setBgmVolume] = useState(() => {
    try {
      const stored = localStorage.getItem('game_bgm_volume');
      return stored ? parseFloat(stored) : 0.3;
    } catch {
      return 0.3;
    }
  });

  const bgmRef = useRef<BgmState>({ howl: null, name: null });
  const sfxHowlRef = useRef<Howl | null>(null);
  const sfxLoadAttempted = useRef(false);
  const initialSfxVolume = useRef(sfxVolume);

  useEffect(() => {
    if (sfxLoadAttempted.current) return;
    sfxLoadAttempted.current = true;

    const howl = new Howl({
      src: ['/audio/sfx.webm', '/audio/sfx.mp3'],
      sprite: SFX_SPRITES,
      volume: initialSfxVolume.current,
      preload: true,
      onloaderror: () => {
        sfxHowlRef.current = null;
      },
    });

    sfxHowlRef.current = howl;

    return () => {
      howl.unload();
    };
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem('game_muted', muted ? '1' : '0');
    } catch { /* noop */ }
    Howler.mute(muted);
  }, [muted]);

  useEffect(() => {
    const bgm = bgmRef.current.howl;
    if (bgm) bgm.volume(bgmVolume);
    try {
      localStorage.setItem('game_bgm_volume', String(bgmVolume));
    } catch { /* noop */ }
  }, [bgmVolume]);

  useEffect(() => {
    const sfx = sfxHowlRef.current;
    if (sfx) sfx.volume(sfxVolume);
    try {
      localStorage.setItem('game_sfx_volume', String(sfxVolume));
    } catch { /* noop */ }
  }, [sfxVolume]);

  useEffect(() => {
    return () => {
      bgmRef.current.howl?.unload();
    };
  }, []);

  const playSfx = useCallback(
    (name: SoundName) => {
      if (muted) return;

      const howl = sfxHowlRef.current;
      if (howl && howl.state() === 'loaded') {
        howl.play(name);
      } else {
        generateSfx(name);
      }
    },
    [muted],
  );

  const playBgm = useCallback(
    (name: BgmName) => {
      const current = bgmRef.current;

      if (current.name === name && current.howl) {
        if (!current.howl.playing()) {
          current.howl.play();
        }
        return;
      }

      current.howl?.fade(current.howl.volume(), 0, 500);
      setTimeout(() => {
        current.howl?.stop();
        current.howl?.unload();
      }, 550);

      const howl = new Howl({
        src: [`/audio/bgm/${name}.webm`, `/audio/bgm/${name}.mp3`],
        volume: 0,
        loop: true,
        preload: true,
        onplay: () => {
          howl.fade(0, bgmVolume, 1000);
        },
        onloaderror: () => { /* noop */ },
      });

      bgmRef.current = { howl, name };
      howl.play();
    },
    [bgmVolume],
  );

  const stopBgm = useCallback(() => {
    const current = bgmRef.current;
    if (!current.howl) return;

    current.howl.fade(current.howl.volume(), 0, 500);
    setTimeout(() => {
      current.howl?.stop();
      current.howl?.unload();
      bgmRef.current = { howl: null, name: null };
    }, 550);
  }, []);

  const toggleMute = useCallback(() => {
    setMuted((prev) => !prev);
  }, []);

  return {
    playSfx,
    playBgm,
    stopBgm,
    muted,
    toggleMute,
    sfxVolume,
    setSfxVolume,
    bgmVolume,
    setBgmVolume,
  };
}
