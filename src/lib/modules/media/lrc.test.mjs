import { test } from "node:test";
import assert from "node:assert/strict";
import { cleanTrack, currentLine, parseLrc } from "./lrc.ts";

test("parses timestamps, fractions and repeated stamps", () => {
  const lines = parseLrc("[ar:x]\n[00:01.50] one\n[00:03.2][01:00.000]chorus\n[00:02] two\nno stamp");
  assert.deepEqual(lines, [
    { ms: 1500, text: "one" },
    { ms: 2000, text: "two" },
    { ms: 3200, text: "chorus" },
    { ms: 60000, text: "chorus" },
  ]);
});

test("finds the current line", () => {
  const lines = parseLrc("[00:01]a\n[00:05]b");
  assert.equal(currentLine(lines, 0), -1);
  assert.equal(currentLine(lines, 1000), 0);
  assert.equal(currentLine(lines, 4999), 0);
  assert.equal(currentLine(lines, 9000), 1);
});

test("cleans video titles for the lyrics lookup", () => {
  assert.deepEqual(cleanTrack("Ed Sheeran - Shape of You (Official Music Video)", "Ed Sheeran"), { track: "Shape of You", artist: "Ed Sheeran" });
  assert.deepEqual(cleanTrack("Shape of You", "Ed Sheeran - Topic"), { track: "Shape of You", artist: "Ed Sheeran" });
  assert.deepEqual(cleanTrack("Song Name [Lyrics] ft. Someone", "ArtistVEVO"), { track: "Song Name", artist: "Artist" });
  assert.deepEqual(cleanTrack("ชื่อเพลง - ศิลปิน【Official MV】", "ค่ายเพลง"), { track: "ศิลปิน", artist: "ชื่อเพลง" });
  assert.deepEqual(cleanTrack("Yellow", "Coldplay"), { track: "Yellow", artist: "Coldplay" });
});
