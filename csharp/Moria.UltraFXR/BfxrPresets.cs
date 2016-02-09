using System;

namespace Moria.UltraFXR
{
	/// <summary>
	/// Presets for BFXR parameters.
	/// </summary>
	public static class BfxrPresets
	{
		private static Random rand = new Random();
		private static double randFloat(double min, double max)
		{
			return min + rand.NextDouble() * (max - min);
		}

		/// <summary>
		/// Generate parameters for a random coin pickup sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters PickupCoin() {
			var p = BfxrParameters.Default();

			p.StartFrequency = randFloat(0.4, 0.9);
			p.SustainTime = randFloat(0, 0.1);
			p.DecayTime = randFloat(0.1, 0.5);
			p.SustainPunch = randFloat(0.3, 0.6);

			if (rand.NextDouble() < 0.5)
			{
				p.ChangeSpeed = randFloat(0.5, 0.7);
				int numer = rand.Next(1, 8);
				int denom = numer + rand.Next(2, 9);
				p.ChangeAmount = (double)numer / (double)denom;
			}

			return p;
		}

		/// <summary>
		/// Generate parameters for a random laser shoot sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters LaserShoot()
		{
			var p = BfxrParameters.Default();

			switch (rand.Next(0, 12) / 5)
			{
				case 0:
					p.Wave = BfxrWave.Square;
					break;
				case 1:
					p.Wave = BfxrWave.Saw;
					break;
				case 2:
					p.Wave = BfxrWave.Sine;
					break;
			}

			p.StartFrequency = randFloat(0.5, 1.0);
			p.MinFrequency = Math.Max(0.2, p.StartFrequency - randFloat(0.2, 0.8));
			p.Slide = randFloat(-0.35, -0.15);

			if (rand.NextDouble() < 0.33)
			{
				p.StartFrequency = randFloat(0, 0.6);
				p.MinFrequency = randFloat(0, 0.1);
				p.Slide = randFloat(-0.65, -0.35);
			}

			if (p.Wave == BfxrWave.Square)
			{
				if (rand.NextDouble() < 0.5)
				{
					p.SquareDuty = randFloat(0, 0.5);
					p.DutySweep = randFloat(0, 0.2);
				}
				else
				{
					p.SquareDuty = randFloat(0.4, 0.9);
					p.DutySweep = randFloat(-0.7, 0);
				}
			}

			p.SustainTime = randFloat(0.1, 0.3);
			p.DecayTime = randFloat(0, 0.4);
			if (rand.NextDouble() < 0.5)
			{
				p.SustainPunch = randFloat(0, 0.3);
			}

			if (rand.NextDouble() < 0.33)
			{
				p.FlangerOffset = randFloat(0, 0.2);
				p.FlangerSweep = randFloat(-0.2, 0);
			}

			if (rand.NextDouble() < 0.5)
			{
				p.HPFilterCutoff = randFloat(0, 0.3);
			}

			return p;
		}

		/// <summary>
		/// Generate parameters for a random explosion sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters Explosion()
		{
			var p = BfxrParameters.Default();

			p.Wave = BfxrWave.Noise;

			if (rand.NextDouble() < 0.5)
			{
				p.StartFrequency = randFloat(0.1, 0.4);
				p.Slide = randFloat(-0.5, -0.1);
			}
			else
			{
				p.StartFrequency = randFloat(0.2, 0.7);
				p.Slide = randFloat(-0.4, -0.2);
			}
			p.StartFrequency = p.StartFrequency * p.StartFrequency;
			if (rand.NextDouble() < 0.2)
			{
				p.Slide = 0;
			}
			if (rand.NextDouble() < 0.33)
			{
				p.RepeatSpeed = randFloat(0.3, 0.8);
			}

			p.SustainTime = randFloat(0.1, 0.4);
			p.DecayTime = randFloat(0, 0.5);
			p.SustainPunch = randFloat(0.2, 0.8);

			if (rand.NextDouble() < 0.5)
			{
				p.FlangerOffset = randFloat(-0.3, 0.6);
				p.FlangerSweep = randFloat(-0.3, 0);
			}

			if (rand.NextDouble() < 0.33)
			{
				p.ChangeSpeed = randFloat(0.6, 0.9);
				p.ChangeAmount = randFloat(-0.8, 0.8);
			}

			return p;
		}

		/// <summary>
		/// Generate parameters for a random powerup sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters Powerup()
		{
			var p = BfxrParameters.Default();

			return p;
		}

		/// <summary>
		/// Generate parameters for a random hit or hurt sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters HitHurt()
		{
			var p = BfxrParameters.Default();

			return p;
		}

		/// <summary>
		/// Generate parameters for a random jump sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters Jump()
		{
			var p = Jump();

			return p;
		}

		/// <summary>
		/// Generate parameters for a random beep or select sound.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters BlipSelect()
		{
			var p = BlipSelect();

			return p;
		}
	}
}
