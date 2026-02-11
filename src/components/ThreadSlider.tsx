import { useMemo, useState } from 'react';

interface ThreadSliderProps {
  value: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}

function debounce<T extends (...args: any[]) => void>(fn: T, delay: number) {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

export function ThreadSlider({ value, onChange, disabled }: ThreadSliderProps) {
  const [localValue, setLocalValue] = useState(value);
  
  const debouncedOnChange = useMemo(
    () => debounce((newValue: number) => onChange(newValue), 300),
    [onChange]
  );

  const handleDecrement = () => {
    const newValue = Math.max(1, localValue - 1);
    setLocalValue(newValue);
    debouncedOnChange(newValue);
  };

  const handleIncrement = () => {
    const newValue = Math.min(8, localValue + 1);
    setLocalValue(newValue);
    debouncedOnChange(newValue);
  };

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-20 h-20 bg-cazz-card border-2 border-cazz-border rounded-lg flex items-center justify-center">
        {/* Number display */}
        <span className="text-2xl font-mono font-bold text-blue-400 tabular-nums z-10">
          {localValue}
        </span>
        
        {/* Dots indicator */}
        <div className="absolute bottom-1 left-1/2 -translate-x-1/2 flex gap-0.5">
          {Array.from({ length: 8 }).map((_, i) => (
            <div
              key={i}
              className={`w-1 h-1 rounded-full ${
                i < localValue ? 'bg-blue-500' : 'bg-cazz-border'
              }`}
            />
          ))}
        </div>

        {/* Decrement button */}
        <button
          onClick={handleDecrement}
          disabled={disabled || localValue <= 1}
          className="absolute left-0 top-0 bottom-0 w-6 flex items-center justify-center text-cazz-muted hover:text-cazz-accent disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          <span className="font-mono text-lg">−</span>
        </button>

        {/* Increment button */}
        <button
          onClick={handleIncrement}
          disabled={disabled || localValue >= 8}
          className="absolute right-0 top-0 bottom-0 w-6 flex items-center justify-center text-cazz-muted hover:text-cazz-accent disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          <span className="font-mono text-lg">+</span>
        </button>
        
        {/* Label above */}
        <span className="absolute -top-6 left-1/2 -translate-x-1/2 text-[10px] font-mono text-cazz-muted uppercase tracking-wider whitespace-nowrap">
          doomscrolling threads
        </span>
      </div>
    </div>
  );
}
