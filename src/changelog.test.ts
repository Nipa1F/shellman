import { describe, expect, it } from 'vitest';
import { bundledChangelog, compareVersions, releaseDate, releaseNotes, releasesSince } from './changelog';

describe('changelog', () => {
  it('turns GitHub generated notes into pull request titles', () => {
    const body = [
      "## What's Changed",
      '* Settings: keep the dialog and its tab list in place by @someone in https://github.com/example/shellman/pull/9',
      '* Faster startup by @other-dev in https://github.com/example/shellman/pull/10',
      '',
      '## New Contributors',
      '* @other-dev made their first contribution in https://github.com/example/shellman/pull/10',
      '',
      '',
      '**Full Changelog**: https://github.com/example/shellman/compare/v0.5.1...v0.5.2',
    ].join('\r\n');
    expect(releaseNotes(body)).toEqual(['Settings: keep the dialog and its tab list in place', 'Faster startup']);
  });

  it('keeps hand-written notes, one change per line', () => {
    expect(releaseNotes('More colours: 15 presets.')).toEqual(['More colours: 15 presets.']);
    expect(releaseNotes('Import from XShell.\nGroups can have subgroups.\n')).toEqual(['Import from XShell.', 'Groups can have subgroups.']);
    expect(releaseNotes('- Dashed item\n* Starred item')).toEqual(['Dashed item', 'Starred item']);
    expect(releaseNotes('')).toEqual([]);
  });

  it('formats release dates', () => {
    expect(releaseDate('2026-10-21T12:00:00Z')).toBe('21 Oct 2026');
    expect(releaseDate('')).toBe('');
    expect(releaseDate('not a date')).toBe('');
  });

  it('compares versions by number', () => {
    expect(compareVersions('0.6.10', '0.6.9')).toBeGreaterThan(0);
    expect(compareVersions('0.6.3', '1.0.0')).toBeLessThan(0);
    expect(compareVersions('0.6.3', '0.6.3')).toBe(0);
  });

  it('lists the releases since the version that ran last, newest first', () => {
    const rel = (version: string) => ({ version, date: '', notes: '' });
    const releases = [rel('0.6.4'), rel('0.6.2'), rel('0.6.3'), rel('0.6.1'), rel('0.5.9')];
    const versions = (last: string, current: string) => releasesSince(releases, last, current).map((r) => r.version);
    // One update, and several skipped ones.
    expect(versions('0.6.3', '0.6.4')).toEqual(['0.6.4']);
    expect(versions('0.6.1', '0.6.4')).toEqual(['0.6.4', '0.6.3', '0.6.2']);
    // The changelog may list releases newer than this build (a test build of an older version).
    expect(versions('0.6.1', '0.6.3')).toEqual(['0.6.3', '0.6.2']);
    // No update, or back to an older version: nothing.
    expect(versions('0.6.4', '0.6.4')).toEqual([]);
    expect(versions('0.6.4', '0.6.2')).toEqual([]);
    // Settings from before the last version was kept: only this release.
    expect(versions('', '0.6.4')).toEqual(['0.6.4']);
    expect(versions('', '0.7.0')).toEqual([]);
  });

  it('reads the bundled changelog defensively', () => {
    expect(bundledChangelog(null)).toEqual({ repo: '', releases: [] });
    expect(bundledChangelog([])).toEqual({ repo: '', releases: [] });
    expect(
      bundledChangelog({
        repo: 'example/shellman',
        releases: [{ version: '0.5.2', date: '2026-10-21T19:33:33Z', notes: 'Fix' }, { version: 3 }, null, { version: '0.5.1' }],
      }),
    ).toEqual({
      repo: 'example/shellman',
      releases: [
        { version: '0.5.2', date: '2026-10-21T19:33:33Z', notes: 'Fix' },
        { version: '0.5.1', date: '', notes: '' },
      ],
    });
  });
});
