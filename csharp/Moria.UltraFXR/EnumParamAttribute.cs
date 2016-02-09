using System;

namespace Moria.UltraFXR
{
	[System.AttributeUsage(System.AttributeTargets.Property)]
	public class EnumParamAttribute : ParamAttribute
	{
		public override object Default { get { return this.DefaultValue; } }
		public object DefaultValue { get; private set; }
		public EnumParamAttribute(int order, string name, object initVal)
			: base(order, name)
		{
			this.DefaultValue = initVal;
		}
	}
}

