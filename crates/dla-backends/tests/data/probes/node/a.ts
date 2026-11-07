import { interval, Observable } from 'rxjs';
import { takeUntil, map } from 'rxjs/operators';
declare function load(): Promise<number>;
export function f(o: Observable<number>) {
  interval(10).subscribe();
  o.subscribe(() => { o.subscribe(); });
  interval(10).pipe(takeUntil(o), map(x => x)).subscribe(() => {});
  load();
}
