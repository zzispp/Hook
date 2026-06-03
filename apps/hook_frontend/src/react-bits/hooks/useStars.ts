import { useState, useEffect } from 'react';

const CACHE_KEY = 'github_stars_cache';
const CACHE_DURATION = 24 * 60 * 60 * 1000;
const DEFAULT_STARS = 33200;
const GITHUB_API_URL = 'https://api.github.com/repos/DavidHDev/react-bits';

function readCachedStars() {
  const cachedData = localStorage.getItem(CACHE_KEY);
  if (!cachedData) return null;

  const { count, timestamp } = JSON.parse(cachedData);
  const isFresh = Date.now() - timestamp < CACHE_DURATION;
  return isFresh && count && count !== 'NAN' ? count : null;
}

async function fetchStarsCount() {
  const response = await fetch(GITHUB_API_URL);
  if (!response.ok) throw new Error(`GitHub stars request failed: ${response.status}`);

  const data = await response.json();
  return data.stargazers_count;
}

export const useStars = () => {
  const [stars, setStars] = useState(DEFAULT_STARS);

  useEffect(() => {
    let cancelled = false;

    async function loadStars() {
      const cached = readCachedStars();
      if (cached) {
        setStars(cached);
        return;
      }

      const count = await fetchStarsCount();
      if (cancelled || !count || count === 'NAN') return;

      localStorage.setItem(CACHE_KEY, JSON.stringify({ count, timestamp: Date.now() }));
      setStars(count);
    }

    loadStars().catch((error) => {
      console.error('Error fetching stars:', error);
    });

    return () => {
      cancelled = true;
    };
  }, []);

  return stars;
};
