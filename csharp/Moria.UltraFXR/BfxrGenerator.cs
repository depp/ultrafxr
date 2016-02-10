using System;

namespace Moria.UltraFXR
{
	/// <summary>
	/// BFXR engine for generating sounds.
	/// </summary>
	public class BfxrGenerator : IGenerator
	{
		private const double MinLength = 0.18;
		private const int LoResNoisePeriod = 8;

		private readonly Random rand;

		// Temporary
		private double period, maxPeriod;
		private double slide, deltaSlide;
		private double squareDuty, dutySweep;
		private int changePeriod, changePeriodTime;
		private double changeAmount, changeAmount2;
		private int changeTime, changeTime2;
		private int changeLimit, changeLimit2;
		private bool changeReached, changeReached2;

		// Permanent
		private double masterVolume;
		private BfxrWave wave;
		private double sustainPunch;
		private double phase;
		private double minFrequency;
		private bool muted;
		private int overtones;
		private double overtoneFalloff;
		private double bitcrushFreq, bitcrushFreqSweep, bitcrushPhase, bitcrushLast;
		private double compressionFactor;
		private bool filters;
		private bool lpFilterOn;
		private double lpFilterPos, lpFilterDeltaPos, lpFilterCutoff, lpFilterDeltaCutoff, lpFilterDamping;
		private double hpFilterPos, hpFilterCutoff, hpFilterDeltaCutoff;
		private double vibratoPhase, vibratoSpeed, vibratoAmplitude;
		private int envelopeStage, envelopeTime, envelopeLength0, envelopeLength1, envelopeLength2, envelopeLength, envelopeFullLength;
		private double envelopeVolume, envelopeInvLength0, envelopeInvLength1, envelopeInvLength2;
		private bool flanger;
		private double flangerOffset, flangerDeltaOffset;
		private int flangerPos;
		private float[] flangerBuffer, noiseBuffer, pinkNoiseBuffer, loResNoiseBuffer;
		private int repeatTime, repeatLimit;

		public int? Length { get { return (int)this.envelopeFullLength; } }

		public BfxrGenerator(BfxrParameters p)
		{
			this.rand = new Random();

			this.period = 100.0 / (p.StartFrequency * p.StartFrequency + 0.001);
			this.maxPeriod = 100.0 / (p.MinFrequency * p.MinFrequency + 0.001);

			this.slide = 1.0 - 0.01 * p.Slide * p.Slide * p.Slide;
			this.deltaSlide = -0.000001 * p.DeltaSlide * p.DeltaSlide * p.DeltaSlide;

			this.squareDuty = 0;
			this.dutySweep = 0;
			if (p.Wave == BfxrWave.Square)
			{
				this.squareDuty = 0.5 - 0.5 * p.SquareDuty;
				this.dutySweep = -0.00005 * p.DutySweep;
			}

			this.changePeriod = (int)(((1 - p.ChangeRepeat) + 0.1) / 1.1 * 20000 + 32);
			this.changePeriodTime = 0;

			this.changeAmount = p.ChangeAmount > 0
				? 1.0 - 0.9 * p.ChangeAmount * p.ChangeAmount
				: 1.0 + 10.0 * p.ChangeAmount * p.ChangeAmount;
			this.changeAmount2 = p.ChangeAmount2 > 0
				? 1.0 - 0.9 * p.ChangeAmount2 * p.ChangeAmount2
				: 1.0 + 10.0 * p.ChangeAmount2 * p.ChangeAmount2;
			this.changeTime = 0;
			this.changeTime2 = 0;
			this.changeLimit = p.ChangeSpeed == 1.0
				? 0
				: (int)(((1.0 - p.ChangeSpeed) * (1.0 - p.ChangeSpeed) * 20000 + 32) *
					(1.0 - p.ChangeRepeat + 0.1) / 1.1);
			this.changeLimit2 = p.ChangeSpeed2 == 1.0
				? 0
				: (int)(((1.0 - p.ChangeSpeed2) * (1.0 - p.ChangeSpeed2) * 20000 + 32) *
					(1.0 - p.ChangeRepeat + 0.1) / 1.1);

			// Total reset
			this.masterVolume = p.MasterVolume * p.MasterVolume;
			this.wave = p.Wave;

			double attackTime = Math.Max(0, p.AttackTime);
			double sustainTime = Math.Max(0.01, p.SustainTime);
			double decayTime = Math.Max(0, p.DecayTime);
			double totalTime = attackTime + sustainTime + decayTime;
			if (totalTime < MinLength)
			{
				double fac = MinLength / totalTime;
				attackTime *= fac;
				sustainTime *= fac;
				decayTime *= fac;
			}
			this.sustainPunch = p.SustainPunch;
			this.phase = 0;

			this.minFrequency = p.MinFrequency;
			this.muted = false;
			this.overtones = (int)(p.Overtones * 10);
			this.overtoneFalloff = p.OvertoneFalloff;
			this.bitcrushFreq = 1 - Math.Pow(p.BitCrush, 1.0 / 3.0);
			this.bitcrushFreqSweep = -0.000015 * p.BitCrushSweep;
			this.bitcrushPhase = 0;
			this.bitcrushLast = 0;

			this.compressionFactor = 1 / (1 + 4 * p.CompressionAmount);
			this.filters = p.LPFilterCutoff != 1.0 || p.HPFilterCutoff != 0.0;

			this.lpFilterOn = p.LPFilterCutoff != 1.0;
			this.lpFilterPos = 0.0;
			this.lpFilterDeltaPos = 0.0;
			this.lpFilterCutoff = 0.1 * p.LPFilterCutoff * p.LPFilterCutoff * p.LPFilterCutoff;
			this.lpFilterDeltaCutoff = 1.0 + 0.0001 * p.LPFilterCutoffSweep;
			this.lpFilterDamping = 1.0 - 5.0 / (1.0 + 20 * p.LPFilterResonance * p.LPFilterResonance) * (0.01 + this.lpFilterCutoff);

			this.hpFilterPos = 0.0;
			this.hpFilterCutoff = 0.1 * p.HPFilterCutoff * p.HPFilterCutoff;
			this.hpFilterDeltaCutoff = 1.0 + 0.003 * p.HPFilterCutoffSweep;

			this.vibratoPhase = 0;
			this.vibratoSpeed = 0.01 * p.VibratoSpeed * p.VibratoSpeed;
			this.vibratoAmplitude = 0.5 * p.VibratoDepth;

			this.envelopeVolume = 0;
			this.envelopeStage = 0;
			this.envelopeTime = 0;
			this.envelopeLength0 = (int)(100000 * attackTime * attackTime);
			this.envelopeLength1 = (int)(100000 * sustainTime * sustainTime);
			this.envelopeLength2 = (int)(100000 * decayTime * decayTime + 10);
			this.envelopeLength = this.envelopeLength0;
			this.envelopeFullLength = this.envelopeLength0 + this.envelopeLength1 + this.envelopeLength2;

			this.envelopeInvLength0 = 1.0 / this.envelopeLength0;
			this.envelopeInvLength1 = 1.0 / this.envelopeLength1;
			this.envelopeInvLength2 = 1.0 / this.envelopeLength2;

			this.flanger = p.FlangerOffset != 0.0 || p.FlangerSweep != 0.0;
			this.flangerOffset = 1020.0 * p.FlangerOffset * p.FlangerOffset * (p.FlangerOffset > 0 ? 1 : -1);
			this.flangerDeltaOffset = 0.2 * p.FlangerSweep * p.FlangerSweep * p.FlangerSweep;
			this.flangerPos = 0;

			this.flangerBuffer = new float[1024];
			this.noiseBuffer = new float[32];
			this.pinkNoiseBuffer = new float[32];
			this.loResNoiseBuffer = new float[32];

			for (int i = 0; i < this.noiseBuffer.Length; i++)
			{
				this.noiseBuffer[i] = 2.0f * (float)this.rand.NextDouble() - 1.0f;
			}
			for (int i = 0; i < this.pinkNoiseBuffer.Length; i++)
			{
				this.pinkNoiseBuffer[i] = 0.0f;
			}
			float val = 0.0f;
			for (int i = 0; i < this.loResNoiseBuffer.Length; i++)
			{
				if ((i % LoResNoisePeriod) == 0)
				{
					val = 2.0f * (float)this.rand.NextDouble() - 1.0f;
				}
				this.loResNoiseBuffer[i] = val;
			}

			this.repeatTime = 0;
			this.repeatLimit = p.RepeatSpeed == 0.0
				? 0
				: (int)((1.0 - p.RepeatSpeed) * (1.0 - p.RepeatSpeed) * 20000) + 32;
		}

		public bool Render(float[] buffer)
		{
			bool finished = false;
			int i;
			for (i = 0; i < buffer.Length && !finished; i++)
			{
				if (this.repeatLimit != 0)
				{
					this.repeatTime++;
					if (this.repeatTime >= this.repeatLimit)
					{
						this.repeatTime = 0;
						// RESET(false)
					}
				}

				this.changePeriodTime++;
				if (this.changePeriodTime >= this.changePeriod)
				{
					this.changeTime = 0;
					this.changeTime2 = 0;
					this.changePeriodTime = 0;
					if (this.changeReached)
					{
						this.period /= this.changeAmount;
						this.changeReached = false;
					}
					if (this.changeReached2)
					{
						this.period /= this.changeAmount2;
						this.changeReached2 = false;
					}
				}

				if (!this.changeReached)
				{
					this.changeTime++;
					if (this.changeTime >= this.changeLimit)
					{
						this.changeReached = true;
						this.period *= this.changeAmount;
					}
				}

				if (this.changeReached2)
				{
					this.changeTime2++;
					if (this.changeTime2 >= this.changeLimit2)
					{
						this.changeReached2 = true;
						this.period *= this.changeAmount2;
					}
				}

				this.slide += this.deltaSlide;
				this.period *= this.slide;

				if (this.period >= this.maxPeriod)
				{
					this.period = this.maxPeriod;
					if (this.minFrequency > 0)
					{
						this.muted = true;
					}
				}

				int periodTemp;
				{
					double p = this.period;
					if (this.vibratoAmplitude > 0.0)
					{
						this.vibratoPhase += this.vibratoSpeed;
						p = this.period * (1.0 + Math.Sin(this.vibratoPhase) * this.vibratoAmplitude);
					}
					periodTemp = Math.Max(8, (int)p);
				}

				if (this.wave == BfxrWave.Square)
				{
					this.squareDuty += this.dutySweep;
					this.squareDuty = Math.Max(0, Math.Min(0.5, this.squareDuty));
				}

				this.envelopeTime++;
				if (this.envelopeTime >= this.envelopeLength)
				{
					this.envelopeTime = 0;
					this.envelopeStage++;
					switch (this.envelopeStage)
					{
						case 1:
							this.envelopeLength = this.envelopeLength1;
							break;
						case 2:
							this.envelopeLength = this.envelopeLength2;
							break;
						case 3:
							this.envelopeLength = int.MaxValue;
							break;
					}
				}

				switch (this.envelopeStage)
				{
					case 0:
						this.envelopeVolume = this.envelopeTime * this.envelopeInvLength0;
						break;
					case 1:
						this.envelopeVolume = 1.0 + (1.0 - this.envelopeTime * this.envelopeInvLength1) * 2.0 * this.sustainPunch;
						break;
					case 2:
						this.envelopeVolume = 1.0 - this.envelopeTime * this.envelopeInvLength2;
						break;
					case 3:
						this.envelopeVolume = 0.0;
						finished = true;
						break;
				}

				int flangerInt = 0;
				if (this.flanger)
				{
					this.flangerOffset += this.flangerDeltaOffset;
					flangerInt = Math.Max(0, Math.Min(this.flangerBuffer.Length - 1, (int)this.flangerOffset));
				}

				if (this.filters && this.hpFilterDeltaCutoff != 0)
				{
					this.hpFilterCutoff *= this.hpFilterDeltaCutoff;
					this.hpFilterCutoff = Math.Max(0.00001, Math.Min(0.1, this.hpFilterCutoff));
				}

				double superSample = 0.0;
				for (int j = 0; j < 8; j++)
				{
					this.phase++;
					if (this.phase >= periodTemp)
					{
						this.phase = this.phase - periodTemp;

						switch (this.wave)
						{
							case BfxrWave.Noise:
								for (var k = 0; k < this.noiseBuffer.Length; k++)
								{
									this.noiseBuffer[k] = 2.0f * (float)rand.NextDouble() - 1.0f;
								}
								break;
							case BfxrWave.Pink:
								for (var k = 0; k < this.pinkNoiseBuffer.Length; k++)
								{
									this.pinkNoiseBuffer[k] = 0.0f;
								}
								break;
							case BfxrWave.Tan:
								float val = 0.0f;
								for (var k = 0; k < this.loResNoiseBuffer.Length; k++)
								{
									if ((i % LoResNoisePeriod) == 0)
									{
										val = 2.0f * (float)this.rand.NextDouble() - 1.0f;
									}
									this.loResNoiseBuffer[k] = val;
								}
								break;
						}
					}

					double sample = 0.0;
					double overtoneStrength = 1.0;
					for (int k = 0; k <= this.overtones; k++)
					{
						int tempPhase = (int)(this.phase * (k + 1)) % periodTemp;
						double relPhase = (double)tempPhase / (double)periodTemp;
						double osample = 0.0;
						switch (this.wave)
						{
							case BfxrWave.Square:
								osample = relPhase < this.squareDuty ? 0.5 : -0.5;
								break;
							case BfxrWave.Saw:
								osample = 1.0 - relPhase * 2.0;
								break;
							case BfxrWave.Sine:
								osample = SineWave(relPhase);
								break;
							case BfxrWave.Noise:
								osample = this.noiseBuffer[(int)(relPhase * this.noiseBuffer.Length) % this.noiseBuffer.Length];
								break;
							case BfxrWave.Triangle:
								osample = Math.Abs(1.0 - relPhase * 2.0) - 1.0;
								break;
							case BfxrWave.Pink:
								osample = this.pinkNoiseBuffer[(int)(relPhase * this.pinkNoiseBuffer.Length) % this.pinkNoiseBuffer.Length];
								break;
							case BfxrWave.Tan:
								osample = Math.Tan(Math.PI * relPhase);
								break;
							case BfxrWave.Whistle:
								osample = SineWave(relPhase) + 0.25 * SineWave(20 * relPhase % 1.0);
								break;
							case BfxrWave.Breaker:
								osample = Math.Abs(1 - 2 * relPhase * relPhase) - 1;
								break;
						}
						sample += overtoneStrength * osample;
						overtoneStrength *= 1 - this.overtoneFalloff;
					}

					if (this.filters)
					{
						double lpFilterOldPos = this.lpFilterPos;
						this.lpFilterCutoff *= this.lpFilterDeltaCutoff;
						this.lpFilterCutoff = Math.Max(0, Math.Min(0.1, this.lpFilterCutoff));
						if (this.lpFilterOn)
						{
							this.lpFilterDeltaPos =
								(this.lpFilterDeltaPos + (sample - this.lpFilterPos) * this.lpFilterCutoff) *
								this.lpFilterDamping;
						}
						else
						{
							this.lpFilterPos = sample;
							this.lpFilterDeltaPos = 0;
						}
						this.lpFilterPos += this.lpFilterDeltaPos;

						this.hpFilterPos = (this.hpFilterPos + this.lpFilterPos - lpFilterOldPos) * (1 - this.hpFilterCutoff);
						sample = this.hpFilterPos;
					}

					if (this.flanger)
					{
						this.flangerBuffer[this.flangerPos & 1023] = (float)sample;
						sample += this.flangerBuffer[(this.flangerPos - flangerInt + 1024) & 1023];
						this.flangerPos = (this.flangerPos + 1) & 1023;
					}

					superSample += sample;
				}

				superSample = Math.Max(-8.0, Math.Min(8.0, superSample));
				superSample *= this.masterVolume * this.envelopeVolume * 0.125;

				this.bitcrushPhase += this.bitcrushFreq;
				if (this.bitcrushPhase > 1)
				{
					this.bitcrushPhase = 0;
					this.bitcrushLast = superSample;
				}
				this.bitcrushFreq = Math.Max(0, Math.Min(1, this.bitcrushFreq + this.bitcrushFreqSweep));
				superSample = this.bitcrushLast;

				superSample = Math.Pow(Math.Abs(superSample), this.compressionFactor) * (superSample > 0 ? 1 : -1);
				if (this.muted)
				{
					superSample = 0;
				}

				buffer[i] = (float)superSample;
			}
			for (; i < buffer.Length; i++)
			{
				buffer[i] = 0.0f;
			}
			return finished;
		}

		private static double SineWave(double phase)
		{
			double x = (phase > 0.5 ? phase - 1.0 : phase) * (2 * Math.PI);
			double y = x < 0
				? 1.27323954 * x + 0.405284735 * x * x
				: 1.27323954 * x - 0.405284735 * x * x;
			return y < 0
				? 0.225 * (+y * y + y) + y
				: 0.225 * (-y * y - y) + y;
		}
	}
}
