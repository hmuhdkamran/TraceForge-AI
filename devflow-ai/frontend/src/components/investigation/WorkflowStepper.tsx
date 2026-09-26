import { workflowSteps, currentStep } from '../../utils/status';
import type { InvestigationStatus } from '../../types';

interface Props {
  status: InvestigationStatus | string;
}

export function WorkflowStepper({ status }: Props) {
  const steps = workflowSteps();
  const current = currentStep(status);

  return (
    <div className="flex items-center gap-1 overflow-x-auto pb-1">
      {steps.map((step, idx) => {
        const done = step.step < current;
        const active = step.step === current;
        const isLast = idx === steps.length - 1;

        return (
          <div key={step.status} className="flex items-center gap-1 shrink-0">
            <div className="flex flex-col items-center">
              <div
                className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold border-2 transition-colors ${
                  done
                    ? 'bg-emerald-500 border-emerald-500 text-white'
                    : active
                    ? 'bg-blue-600 border-blue-600 text-white'
                    : 'bg-white border-gray-300 text-gray-400'
                }`}
              >
                {done ? '✓' : step.step}
              </div>
              <div className={`text-xs mt-1 text-center max-w-16 leading-tight ${active ? 'text-blue-600 font-medium' : 'text-gray-400'}`}>
                {step.label}
              </div>
            </div>
            {!isLast && (
              <div className={`h-0.5 w-6 mb-4 ${done ? 'bg-emerald-400' : 'bg-gray-200'}`} />
            )}
          </div>
        );
      })}
    </div>
  );
}
